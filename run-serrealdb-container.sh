#!/bin/bash

sudo docker run --rm --pull always -p '127.0.0.1:28000:8000' surrealdb/surrealdb:latest start
