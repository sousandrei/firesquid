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

- REST API for interactions
- Multiple vms from the same kernel
- Lightweight
- Customizable

Upcoming:

- Networking
- Proper package release and distribution
- Choices between which kernel to use for which machine

## <a name="help-wanted"></a> Assets 📦

Here are some assets to get you started. The default folder for assets is just called `assets` in the same folder as FireSquid.

[Linux Kernel 5.9-rc2][kernel] compiled with firecracker recommended settings

[sample rootfs.ext4][rootfs] build from the `node-alpine` docker image. Boots, sleeps 3 seconds and then powers off.

[firecracker][firecracker] releases, grab one accordingly

## <a name="help-wanted"></a> Help wanted 🤝

All contributions are welcome!

If you are using or plan to use Firecracker or FireSquid please don't hesitate to reach out so we can foster a collaboration ecosystem around Firecracker.

## <a name="sponsors"></a> Sponsors ❤️

you?

## <a name="license"></a> License

See [LICENSE](https://github.com/sousandrei/firesquid/blob/master/LICENSE) for more details.

[rootfs]: https://storage.googleapis.com/firesquid/rootfs.ext4
[kernel]: https://storage.googleapis.com/firesquid/vmlinux
[firecracker]: https://github.com/firecracker-microvm/firecracker/releases
