#!/bin/bash

sudo docker run --rm --pull always -p '127.0.0.1:8007:8000' surrealdb/surrealdb:latest start
