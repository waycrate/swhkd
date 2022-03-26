#!/usr/bin/env bash

killall swhks

swhks & pkexec swhkd
