#!/usr/bin/env sh

killall swhks

swhks & pkexec swhkd
