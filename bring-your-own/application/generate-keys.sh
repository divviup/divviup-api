#
# This shell script is written to be as portable as possible. It even
# avoids defining any shell functions, as the work to be done is quite
# straightforward.
#
# Each generated key will be one of the following types:
#
#       Type 1: 128 random bits (16 random bytes) written in
#               URL-safe Base64 with no padding.
#
#       Type 2: 256 random bits (32 random bytes) written in
#               URL-safe Base64 with no padding.
#

chmod 600 .env || exit $?

tmp=generate-keys.sh.tmp

to_url_safe='
  s|=||g
  s|+|-|g
  s|/|_|g
'

# DATABASE_ENCRYPTION_KEYS: A single type 1 key.
head -c 16 /dev/urandom >${tmp?}1 || exit $?
base64 -w 0 ${tmp?}1 >${tmp?}2 || exit $?
x=`sed "${to_url_safe?}" ${tmp?}2` || exit $?
sed -i.bak "
  /^#DATABASE_ENCRYPTION_KEYS=/ {
    s|=.*|=${x?}|
    s|#||
  }
" .env || exit $?

# POSTGRES_PASSWORD: A single type 1 key.
head -c 16 /dev/urandom >${tmp?}1 || exit $?
base64 -w 0 ${tmp?}1 >${tmp?}2 || exit $?
x=`sed "${to_url_safe?}" ${tmp?}2` || exit $?
sed -i.bak "
  /^#POSTGRES_PASSWORD=/ {
    s|=.*|=${x?}|
    s|#||
  }
" .env || exit $?

# SESSION_SECRETS: A single type 2 key.
head -c 32 /dev/urandom >${tmp?}1 || exit $?
base64 -w 0 ${tmp?}1 >${tmp?}2 || exit $?
x=`sed "${to_url_safe?}" ${tmp?}2` || exit $?
sed -i.bak "
  /^#SESSION_SECRETS=/ {
    s|=.*|=${x?}|
    s|#||
  }
" .env || exit $?

rm -f ${tmp?}* || exit $?
