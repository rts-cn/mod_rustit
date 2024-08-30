# 说明
这个一个使用rust语言开发FreeSWITCH模块示例

# 已知问题
## CentOS 7默认的 Glibc的版本过低会导致Rust开发的模块无法使用
详情查看：
https://github.com/rust-lang/rust/issues/91979
Segmentation fault when thread using dynamically loaded Rust library exits

按下面的步骤升级glibc版本到2.24即可解决
``` 
sudo yum install gcc make -y
curl -O https://mirrors.tuna.tsinghua.edu.cn/gnu/glibc/glibc-2.24.tar.gz
tar zxf glibc-2.24.tar.gz
cd glibc-2.24/
mkdir out && cd out
../configure --prefix=/usr
make -j$(nproc --all)
make install
```