#!/bin/bash
#
# A utility script for downloading the latest nightly build of Rust,
# uploading it to S3, and updating your RustConfig file.  This requires the
# official 'aws' command line tool, which can be installed and configured
# as follows:
#
#   sudo pip install awscli
#   aws configure
#
# To run this script, first create an S3 bucket, and then run:
#
#   ./update-bin my-bucket-name

# Quit on the first error we encounter.
set -e

# Make sure we were passed an S3 bucket.
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <S3 bucket name>" 1>&2
    exit 1
else
    BUCKET="$1"
fi

# Format today's date appropriately.
DATE=`date '+%Y-%m-%d'`

# The names of our nightly tarballs.
RUST_TARBALL=rust-nightly-x86_64-unknown-linux-gnu.tar.gz
CARGO_TARBALL=cargo-nightly-x86_64-unknown-linux-gnu.tar.gz

# Download our tarballs.
echo "-----> Fetching nightly builds"
rm -f "$RUST_TARBALL" "$CARGO_TARBALL"
curl -O "https://static.rust-lang.org/dist/$RUST_TARBALL"
curl -O "https://static.rust-lang.org/cargo-dist/$CARGO_TARBALL"

# Upload our tarballs to S3.
echo "-----> Uploading to S3"
aws s3 cp "$RUST_TARBALL" "s3://$BUCKET/$DATE/" --acl public-read
aws s3 cp "$CARGO_TARBALL" "s3://$BUCKET/$DATE/" --acl public-read

# Updating RustConfig.
echo "-----> Uploading RustConfig"
cat <<EOF > RustConfig
URL="https://s3.amazonaws.com/$BUCKET/$DATE/$RUST_TARBALL"
VERSION="$DATE"

CARGO_URL="https://s3.amazonaws.com/$BUCKET/$DATE/$CARGO_TARBALL"
CARGO_VERSION="$DATE"
EOF

echo "-----> Cleaning up"
rm -f "$RUST_TARBALL" "$CARGO_TARBALL"
