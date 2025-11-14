# This file configures the S3 remote backend for Terraform state.
# The configuration for this backend is provided during initialization
# via the `-backend-config` flag in the `terraform init` command.
# This allows the backend to be configured dynamically in different environments
# (e.g., local development vs. GitHub Actions).

terraform {
  backend "s3" {}
}
