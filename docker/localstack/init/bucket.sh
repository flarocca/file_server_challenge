#!/usr/bin/env bash
set -euo pipefail

echo "Creating S3 bucket: file-server"
awslocal s3 mb s3://file-server || true

