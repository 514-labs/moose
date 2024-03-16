#!/bin/bash
kubectl create secret generic redpanda-password --from-literal=password=$1