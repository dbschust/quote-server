#!/bin/sh
PW=`cat secrets/reg_password.txt`
CREDS="{
  \"email\": \"dschust@pdx.edu\",
  \"full_name\": \"Daniel Schuster\",
  \"password\": \"$PW\"
}"

ACCESS_TOKEN=`curl -s -X POST -H "Content-type: application/json" \
     -d "$CREDS" \
     http://localhost:3000/api/v1/register | jq .access_token | sed 's/"//g'`

QUOTE='{
  "words": "A king is he that can hold his own or else his title is vain.",
  "id": "tolkien-king",
  "source": "The Silmarillion",
  "tags": [
    "king", "life", "inspiration"
  ],
  "author": "J.R.R. Tolkien"
}'

curl -X POST -H "Content-type: application/json"  \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     -d "$QUOTE" http://localhost:3000/api/v1/add-quote