<p align="center">
  <a href="#">
      <img src="https://storage.googleapis.com/firesquid/firesquid.svg" alt="Logo" width="600" height="180">
  </a>

  <p align="center">
    Simple AWS Firecracker orchestrator
    <br />
    <br />
    <a href="https://github.com/sousandrei/firesquid/issues/new">Report Bug or Request Feature</a>
  </p>
</p>

<br />

[![Build Status](https://github.com/sousandrei/firesquid/workflows/Main/badge.svg)](https://github.com/sousandrei/firesquid/actions)

## Table of Contents

- [About the project](#about)
- [Features](#features)
- [Assets](#assets)
- [Help Wanted](#help-wanted)
- [Sponsors](#sponsors)
- [License](#license)

## <a name="about"></a> About the Project

FireSquid is a simple [AWS Firecracker](https://firecracker-microvm.github.io/) orchestrator.
It abstracts the hard part making it a breeze to spawn your very own fleet of micro-vms!

## <a name="features"></a> Features

- REST API
- Multiple vms from the same kernel
- Lightweight
- Customizable

Upcoming:

- Fine grained networking
- Package release and distribution
- Choices between which kernel to use for which machine

## <a name="help-wanted"></a> Assets 📦

Here are some assets to get you started. The default folder for assets is just called `assets` in the same folder as FireSquid.

Grab the assets listed in the official firecracker guide

```
ARCH="$(uname -m)"

mkdir assets

# Download a linux kernel binary
wget https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.6/${ARCH}/vmlinux-5.10.198 -P assets/vmlinux

# Download a rootfs
wget https://s3.amazonaws.com/spec.ccfc.min/firecracker-ci/v1.6/${ARCH}/ubuntu-22.04.ext4 -P assets/rootfs.ext4

# Download the official firecracker binary
release_url="https://github.com/firecracker-microvm/firecracker/releases"
latest=$(basename $(curl -fsSLI -o /dev/null -w  %{url_effective} ${release_url}/latest))
curl -L ${release_url}/download/${latest}/firecracker-${latest}-${ARCH}.tgz \
| tar -xz release-${latest}-${ARCH}/firecracker-${latest}-${ARCH} --strip-components=1

# Rename the binary to "firecracker"
mv firecracker-${latest}-${ARCH} assets/firecracker
```

## <a name="help-wanted"></a> Help wanted 🤝

All contributions are welcome!

If you are using or plan to use Firecracker or FireSquid please don't hesitate to reach out so we can foster a collaboration ecosystem around Firecracker.

## <a name="sponsors"></a> Sponsors ❤️

you?

## <a name="license"></a> License

See [LICENSE](https://github.com/sousandrei/firesquid/blob/master/LICENSE) for more details.
