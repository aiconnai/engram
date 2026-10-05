#!/bin/bash
# Option E premise, two real uids: can attacker uid 1001 rename-swap the
# victim's (uid 1000) database directory under different ancestor modes?
useradd -u 1000 victim >/dev/null 2>&1; useradd -u 1001 attacker >/dev/null 2>&1
try() { # $1 = ancestor mode
  rm -rf /srv/demo; mkdir -p /srv/demo; chmod "$1" /srv/demo
  install -d -o 1000 -g 1000 -m 700 /srv/demo/engram
  setpriv --reuid=1001 --regid=1001 --clear-groups mv /srv/demo/engram /srv/demo/parked 2>/tmp/err
  if [ $? -eq 0 ]; then r="SWAPPED"; else r="refused ($(sed 's/.*: //' /tmp/err))"; fi
  echo "ancestor mode $1 (root-owned): attacker rename of victim dir -> $r"
}
try 755; try 1777; try 777
