#!/usr/bin/env python3
"""Save and restore the paths selected by prepare-rootfs.sh."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def overlaps(left, right):
    return left == right or left in right.parents or right in left.parents


def present(path):
    return os.path.lexists(path)


def physical(root, name):
    path = root / name
    parent = path.parent.resolve()
    if not parent.is_relative_to(root):
        raise ValueError(f"path leaves ROOTFS: {name}")
    return (parent / path.name).relative_to(root).as_posix()


def inventory(root, scope):
    return [name for name in scope if present(root / name)]


def hash_xattrs(digest, root, path):
    # Filesystems can enumerate extended attributes in different orders.
    # Linux POSIX ACLs are extended attributes too.
    for name in sorted(os.listxattr(path, follow_symlinks=False)):
        value = os.getxattr(path, name, follow_symlinks=False)
        digest.update(os.fsencode(path.relative_to(root)) + b"\0")
        digest.update(os.fsencode(name) + b"\0")
        digest.update(len(value).to_bytes(8, "big"))
        digest.update(value)
    if not path.is_symlink() and path.is_dir():
        for child in sorted(path.iterdir()):
            hash_xattrs(digest, root, child)


def tree_hash(root, paths):
    # Hash names, contents, links, owners, modes and directories in a stable
    # order. Ignore filesystem timestamp precision.
    with tempfile.TemporaryFile() as names:
        names.write(b"".join(os.fsencode(name) + b"\0" for name in paths))
        names.seek(0)
        with subprocess.Popen(
            ["tar", "--sort=name", "--format=gnu", "--mtime=@0", "--numeric-owner",
             "-cf", "-", "-C", root, "--null", "-T", "-"],
            stdin=names, stdout=subprocess.PIPE, env=dict(os.environ, LC_ALL="C")
        ) as archive:
            digest = hashlib.file_digest(archive.stdout, "sha256")
            if archive.wait():
                raise ValueError(f"could not hash {root}")
    for name in paths:
        hash_xattrs(digest, root, root / name)
    return digest.hexdigest()


def sync(src, dst, paths=None, verify=False):
    command = ["rsync", "-aHAX", "--numeric-ids", "--checksum", "--delete", "--force"]
    names = None
    if paths is not None:
        command += ["-r", "--relative", "--no-implied-dirs", "--from0", "--files-from=-"]
        names = b"".join(os.fsencode(name) + b"\0" for name in paths)
    if verify:
        command += ["--dry-run", "--itemize-changes"]
    result = subprocess.run(
        [*command, "--", f"{src}/", f"{dst}/"], input=names,
        stdout=subprocess.PIPE, check=True
    )
    if verify and result.stdout:
        raise ValueError(f"restored files differ in {dst}:\n" + os.fsdecode(result.stdout))


def verify_backup(backup, identity):
    data = (backup / "manifest.json").read_bytes()
    if hashlib.sha256(data).hexdigest() != (backup / "manifest.sha256").read_text().strip():
        raise ValueError("backup manifest checksum mismatch")
    manifest = json.loads(data)
    if manifest["targets"] != identity:
        raise ValueError("backup belongs to different root or TFTP paths")
    for tree, digest in manifest["sha256"].items():
        paths = manifest["present"] if tree == "root" else ["."]
        if tree_hash(backup / tree, paths) != digest:
            raise ValueError(f"{tree} backup checksum mismatch")
    return manifest


def save(root, tftp, backup, identity, new_tree, paths):
    tree = physical(root, new_tree)
    files = {physical(root, name) for name in paths}
    plan = {"files": sorted(files), "new_tree": tree}
    if backup.exists():
        manifest = verify_backup(backup, identity)
        if manifest["plan"] != plan:
            raise ValueError("root paths changed; restore this backup before starting a new one")
        return

    paths = {Path(name) for name in files}
    if not present(root / tree):
        # The helper can create an entire initramfs config tree on the first run.
        paths.add(Path(tree))
    scope = sorted(path.as_posix() for path in paths if paths.isdisjoint(path.parents))
    existing = inventory(root, scope)
    parents = {parent for name in scope for parent in Path(name).parents}
    missing_parents = sorted(p.as_posix() for p in parents if not present(root / p))
    manifest = {
        "targets": identity, "plan": plan, "scope": scope, "present": existing,
        "missing_parents": missing_parents, "sha256": {},
    }
    stage = Path(tempfile.mkdtemp(prefix=backup.name + ".", dir=backup.parent))
    try:
        for tree, src, members in (("root", root, existing), ("tftp", tftp, None)):
            if src is None:
                continue
            dst = stage / tree
            dst.mkdir()
            sync(src, dst, members)
            all_paths = members if members is not None else ["."]
            sync(src, dst, members, verify=True)
            digest = tree_hash(src, all_paths)
            if tree_hash(dst, all_paths) != digest:
                raise ValueError(f"{tree} backup differs from the source")
            manifest["sha256"][tree] = digest
        data = (json.dumps(manifest, indent=2) + "\n").encode()
        (stage / "manifest.json").write_bytes(data)
        (stage / "manifest.sha256").write_text(hashlib.sha256(data).hexdigest() + "\n")
        stage.rename(backup)
    finally:
        if stage.exists():
            shutil.rmtree(stage)


def restore(root, tftp, backup, identity):
    manifest = verify_backup(backup, identity)
    scope = manifest["scope"]
    # Manifest paths use physical parents. Refuse a redirected parent before
    # writing or removing anything.
    for name in scope:
        if physical(root, name) != str(Path(name)):
            raise ValueError(f"root path was redirected: {name}")

    existing = manifest["present"]
    for name in set(inventory(root, scope)) - set(existing):
        path = root / name
        if path.is_dir() and not path.is_symlink():
            shutil.rmtree(path)
        else:
            path.unlink()
    sync(backup / "root", root, existing)
    for name in sorted(manifest["missing_parents"], key=lambda p: len(Path(p).parts), reverse=True):
        path = root / name
        if path.is_dir() and not any(path.iterdir()):
            path.rmdir()
    if inventory(root, scope) != existing or tree_hash(root, existing) != manifest["sha256"]["root"]:
        raise ValueError("restored root checksum mismatch")
    sync(backup / "root", root, existing, verify=True)

    if tftp is not None:
        sync(backup / "tftp", tftp)
        if tree_hash(tftp, ["."]) != manifest["sha256"]["tftp"]:
            raise ValueError("restored TFTP checksum mismatch")
        sync(backup / "tftp", tftp, verify=True)


def main(action, root, tftp, backup, *args):
    root, tftp = Path(root).resolve(), Path(tftp).resolve() if tftp else None
    # Resolve the parent, but leave the final backup name intact.
    backup_path = Path(backup)
    backup = backup_path.parent.resolve() / backup_path.name
    if backup.is_symlink():
        raise ValueError("BACKUP_DIR must not be a symlink")
    for target in (root, tftp):
        if target is not None and (
            not target.is_dir() or os.path.samefile(target, "/") or overlaps(target, backup)
        ):
            raise ValueError("ROOTFS and TFTP_DIR must be directories outside BACKUP_DIR and /")
    if tftp is not None and overlaps(root, tftp):
        raise ValueError("ROOTFS and TFTP_DIR must be separate trees")
    identity = {"root": str(root), "tftp": str(tftp) if tftp is not None else None}
    if action == "validate":
        print(backup)
    elif action == "save":
        new_tree, *paths = args
        save(root, tftp, backup, identity, new_tree, paths)
        print(f"verified backup at {backup}")
    elif action == "restore":
        restore(root, tftp, backup, identity)
        print(f"restored and verified {backup}")
    else:
        raise ValueError(f"unknown backup action: {action}")


if __name__ == "__main__":
    try:
        main(*sys.argv[1:])
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
