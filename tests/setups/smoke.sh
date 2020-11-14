#! /bin/sh

set -o errexit

pass init "${PASSWORD_STORE_KEY}"

cat <<EOF | pass insert --multiline Manufacturers/Yoyodyne
all1the%fancy@panels+are;for<me

user: laforge
EOF

cat <<EOF | pass insert --multiline Phone
PIN: 1701

Pattern:
O--O--5
|  |  |
O--4--3
|  |  |
O--1--2
EOF

cat <<EOF | pass insert --multiline Manufacturers/StrutCo
i*aint*got*no*tape
EOF

cat <<EOF | pass insert --multiline Manufacturers/Sokor
pum-yIghoSQo'
Better not tell Picard about this.
EOF

cat <<EOF | pass insert --multiline Entertainment/Holo\ Deck/Broht\ \&\ Forrester
fun-times1337
username: geordi
EOF
