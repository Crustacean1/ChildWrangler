#!/bin/sh

set -e 

envsubst /config.template > /config
gammu-smsd -c /config

