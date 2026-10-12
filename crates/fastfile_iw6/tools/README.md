`src/layout.rs` is generated from x64-zt's `src/zonetool/zonetool/iw6/structs.hpp`
(https://github.com/Joelrau/x64-zt) with an MSVC x64 record layout:

```sh
sed 's/#include <d3d11.h>/#include "shim.h"/' structs.hpp > cc/structs.hpp   # shim: std:: ints, D3D11 forward decls, DXGI_FORMAT, assert_sizeof
# all.cpp: `extern "C" const unsigned long long sz_N = sizeof(<every struct/union>);`
clang++ -std=c++20 --target=x86_64-pc-windows-msvc -fms-extensions -fsyntax-only \
    -Xclang -fdump-record-layouts all.cpp > layouts.txt
python3 genlayout.py layouts.txt ../src/layout.rs
```
