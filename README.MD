CentOS 7 glibc <= 2.17 
Segmentation fault when thread using dynamically loaded Rust library exits

https://github.com/rust-lang/rust/issues/91979

sudo yum install gcc make -y
curl -O https://mirrors.tuna.tsinghua.edu.cn/gnu/glibc/glibc-2.24.tar.gz
tar zxf glibc-2.24.tar.gz
cd glibc-2.24/
mkdir out && cd out
../configure --prefix=/usr
make -j$(nproc --all)
make install

ldd --version
ldd (GNU libc) 2.24
Copyright (C) 2016 Free Software Foundation, Inc.
This is free software; see the source for copying conditions.  There is NO
warranty; not even for MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
Written by Roland McGrath and Ulrich Drepper.