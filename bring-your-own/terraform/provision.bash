set -E -e -u -o pipefail || exit $?
trap exit ERR

set -x

#-----------------------------------------------------------------------
# Sleep a bit before doing anything else
#-----------------------------------------------------------------------
#
# Terraform sometimes gets into the host so fast that "apt-get update"
# fails with weird "No such file or directory" errors. Doing a sleep
# before doing anything else seems to help.
#

sleep 30

#-----------------------------------------------------------------------
# Make apt-get noninteractive
#-----------------------------------------------------------------------

DEBIAN_FRONTEND=noninteractive
readonly DEBIAN_FRONTEND
export DEBIAN_FRONTEND

#-----------------------------------------------------------------------
# Install some packages
#-----------------------------------------------------------------------

sudo apt-get -q -y update

sudo apt-get -q -y install \
  bash \
  jq \
;

#-----------------------------------------------------------------------
# Install Docker
#-----------------------------------------------------------------------

curl -L -S -f -s https://get.docker.com/ | sudo sh

x=$(sed -n '/^docker:/ p' /etc/group)
if [[ ! $x ]]; then
  sudo groupadd docker
fi

sudo usermod -G docker -a $USER

#-----------------------------------------------------------------------
# Start the Docker Compose deployment
#-----------------------------------------------------------------------

sg docker 'docker compose up -d'
