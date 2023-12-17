ARCH="$(uname -m)"

mkdir assets

# Download a linux kernel binary
wget https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.6/${ARCH}/vmlinux-5.10.198 -O assets/vmlinux

# Download a rootfs
wget https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.6/${ARCH}/ubuntu-22.04.ext4 -O assets/rootfs.ext4

# Download the official firecracker binary
release_url="https://github.com/firecracker-microvm/firecracker/releases"
latest=$(basename $(curl -fsSLI -o /dev/null -w  %{url_effective} ${release_url}/latest))
curl -L ${release_url}/download/${latest}/firecracker-${latest}-${ARCH}.tgz \
| tar -xz release-${latest}-${ARCH}/firecracker-${latest}-${ARCH} --strip-components=1

# Rename the binary to "firecracker"
mv firecracker-${latest}-${ARCH} assets/firecracker