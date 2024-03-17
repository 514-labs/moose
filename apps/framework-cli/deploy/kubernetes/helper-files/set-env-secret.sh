#!/bin/bash
# Helper script to set a secret in the kubernetes cluster using the environment name and secret value
# Example: ./set-env-secret.sh MOOSE_CLICKHOUSE_CONFIG__DB_NAME hosteddb
if [ -z "$1" ]
then
      echo "You must specify the environment key name as the first argument. Example: ./set-env-secret.sh MOOSE_CLICKHOUSE_CONFIG__DB_NAME hosteddb"
      exit 1
fi

if [ -z "$2" ]
then
      echo "You must specify the secret value as the second argument. Example: ./set-env-secret.sh MOOSE_CLICKHOUSE_CONFIG__DB_NAME hosteddb"
      exit 1
fi

lc_str=${1,,}
lc_str=${lc_str//_/-}

kubectl delete secret "sn-${lc_str}"
kubectl create secret generic "sn-${lc_str}" --from-literal="sk-${lc_str}"=$2