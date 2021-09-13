#!/bin/bash
anchor build &&\
anchor deploy &&\
ts-node scripts/localnet-migrate.ts
