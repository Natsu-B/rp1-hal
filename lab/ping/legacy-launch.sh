#!/bin/sh
# Open the schema-2 reservation before starting the matching older client.
set -eu
if [ "$#" -lt 3 ]; then
    echo 'usage: legacy-launch.sh LINUX_PING irq|poll OLD_APP [ARG...]' >&2
    exit 2
fi
launcher=$1
profile=$2
shift 2
case $profile in
    irq|poll) ;;
    *) echo 'legacy launcher requires a schema-2 profile' >&2; exit 2 ;;
esac
"$launcher" --test "$profile" ready
exec "$@"
