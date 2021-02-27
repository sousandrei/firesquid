# Building the Linux Kernel for Firecracker

## Contents

- [Summary](#summary)
- [Prerequisites](#prerequisites)
- [Getting the kernel code from CDN](#getting-the-kernel-code-from-cdn)
- [Getting the kernel code from Github](#getting-the-kernel-code-from-github)
- [Verifying the kernel code](#verifying-the-kernel-code)
- [Building the kernel](#building-the-kernel)

## Summary

A Firecracker VM needs two files: a rootfs (boot drive) and a kernel.

You can find the documentation on the rootfs [here](docs/building-rootfs) and Firecracker's official docs on how to do the same process [here](firecracker-guide)

tl;dr

- download linux kernel code from their repo
- get firecrackers recommended config for building the kernel
- build the kernel

You have two ways of obtaining the code. The usual signed release from linux's CDN or the github mirror.

If you prefer the github way, please skip the verification step

## Prerequisites

Depending on the operation system you are trying to do the compilation, the prerequisites will vary. You can check out the complete documentation of what's neede [here](linux-prereq)

On a debian based machine the following command should install/update everything you will need. If something is missing, kindly open an issue here so we can fix it!

```
sudo apt-get install git fakeroot build-essential ncurses-dev xz-utils libssl-dev bc flex libelf-dev bison
```

We are also using `gunzip` and `unxz` here so make sure you have them.

For all this guide we will assume you are running everything from an empty folder. Since this process is done once, running it separately in a temporary folder is recommended for ease of cleanup.

## Getting the kernel code - CDN

The kernel code usually lives in [linux's official CDN](cdn).

At the time of writing, the latest version is `5.9.1`, so we are going to use it for this guide. Feel free to test different versions

We start by getting the code and it's signature with a simple curl, the we uncompress it!

for the `gz` version

```
curl -O https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.1.tar.gz
gunzip linux-5.9.1.tar.gz
```

for the `xz` version

```
curl -O https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.1.tar.xz
unxz linux-5.9.1.tar.xz
```

The signature is created from the tarball itself so we can use it for both compression options!

```
curl -O https://cdn.kernel.org/pub/linux/kernel/v5.x/linux-5.9.1.tar.sign
```

## Verifying the kernel code

Verifying the code assures us that we are using untampered code from the source we are getting, in this case, the linux kernel official CDN.

First we get the keys from Linus Torvalds and Greg Kroah-Hartman, the usual people who sign this packages

```
gpg2 --locate-keys torvalds@kernel.org gregkh@kernel.org
```

Then we run `gpg2` to make sure everything here is setup correctly.

```
gpg2 --verify linux-5.9.1.tar.sign
```

This will output something along the lines of the following, with `<KEY>` being a value that we are going to re-use here.

```
gpg: assuming signed data in 'linux-5.9.1.tar'
gpg: Signature made lör 17 okt 2020 08:33:43 CEST
gpg:                using RSA key 647F28654894E3BD457199BE38DBBDC86092693E
gpg: Good signature from "Greg Kroah-Hartman <gregkh@kernel.org>" [unknown]
gpg: WARNING: This key is not certified with a trusted signature!
gpg:          There is no indication that the signature belongs to the owner.
Primary key fingerprint: 647F 2865 4894 E3BD 4571  99BE 38DB BDC8 6092 693E
```

In this case we can wee that Greg signed the release but we can't trust his key yet, so we run

```
gpg2 --tofu-policy good 647F28654894E3BD457199BE38DBBDC86092693E
```

Now we run the signature check again

```
gpg2 --trust-model tofu --verify linux-5.9.1.tar.sign
```

And with this output we can finally trust this package!

```
gpg: assuming signed data in 'linux-5.9.1.tar'
gpg: Signature made lör 17 okt 2020 08:33:43 CEST
gpg:                using RSA key 647F28654894E3BD457199BE38DBBDC86092693E
gpg: Good signature from "Greg Kroah-Hartman <gregkh@kernel.org>" [full]
gpg: gregkh@kernel.org: Verified 1 signatures in the past 0 seconds.  Encrypted
     0 messages.
```

Now let's extract the code to prepare for the building step

```
tar -xvf linux-5.9.1.tar
```

## Getting the kernel code - Github

There is also a mirror being hosted on the official [linux repository on github](github-mirror).

If you prefer to use this source, you can ignore the previous steps and directly clone it
At the time of writing, the lastest github version is `5.9`, so checkout to it's tag

```
git clone git@github.com:torvalds/linux.git
git checkout v5.9
```

## Building the kernel

Go into the folder we created earlier, be it `linux` from the github clone or `linux-5.9.1` from the signed artifact

First let's grab the recommended settings for the kernel from the Firecracker official repository.

for x86_x64

```
curl https://raw.githubusercontent.com/firecracker-microvm/firecracker/master/resources/microvm-kernel-x86_64.config -o .config
```

for arm64

```
curl https://raw.githubusercontent.com/firecracker-microvm/firecracker/master/resources/microvm-kernel-arm64.config -o .config
```

Now since this was build using `4.20` and we are using `5.9.1`, there will be differences. So we open the config menu to load the recommended ones and merge with the default new configuration options.

```
make menuconfig
```

Now in the menu make sure to load the `.config` and save it again, then you can exit.

Finally we can build our beloved `vmlinux` artifact!

simple make

```
make vmlinux
```

or, make using all cores in the machine

```
make -j$(nproc) vmlinux
```

Now there is a file called `vmlinux` in your folder, You can grab it and delete this temporary folder!

Simple right?!

[recommended-settings]:[https://github.com/firecracker-microvm/firecracker/blob/master/resources/microvm-kernel-x86_64.config]
[firecracker-guide]: [https://github.com/firecracker-microvm/firecracker/blob/master/docs/rootfs-and-kernel-setup.md]
[linux-prereq]: [https://www.kernel.org/doc/html/v4.14/process/changes.html]
[cdn]: [https://cdn.kernel.org/pub/linux/kernel]
[github-mirror]: [https://github.com/torvalds/linux]
