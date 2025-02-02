#!/bin/sh

export DOLLAR="$" && envsubst < /etc/nginx/nginx.conf.template > /etc/nginx/nginx.conf
cat /etc/nginx/nginx.conf

nginx -g 'daemon off;'