#pragma once
namespace std { typedef signed char int8_t; typedef unsigned char uint8_t; typedef short int16_t; typedef unsigned short uint16_t; typedef int int32_t; typedef unsigned int uint32_t; typedef long long int64_t; typedef unsigned long long uint64_t; typedef unsigned long long size_t; }
typedef signed char int8_t; typedef unsigned char uint8_t; typedef short int16_t; typedef unsigned short uint16_t; typedef int int32_t; typedef unsigned int uint32_t; typedef long long int64_t; typedef unsigned long long uint64_t; typedef unsigned long long size_t;
#define offsetof(T,F) __builtin_offsetof(T,F)
#define assert_sizeof(T, S) static_assert(sizeof(T) == (S), #T)
#define assert_offsetof(T, F, O) static_assert(offsetof(T, F) == (O), #T "." #F)
typedef enum DXGI_FORMAT { DXGI_FORMAT_UNKNOWN = 0 } DXGI_FORMAT;
struct ID3D11Device; struct ID3D11DeviceChild; struct ID3D11Resource; struct ID3D11Buffer; struct ID3D11Texture1D; struct ID3D11Texture2D; struct ID3D11Texture3D;
struct ID3D11ShaderResourceView; struct ID3D11UnorderedAccessView; struct ID3D11RenderTargetView; struct ID3D11DepthStencilView;
struct ID3D11VertexShader; struct ID3D11PixelShader; struct ID3D11HullShader; struct ID3D11DomainShader; struct ID3D11ComputeShader; struct ID3D11GeometryShader;
struct ID3D11InputLayout; struct ID3D11BlendState; struct ID3D11DepthStencilState; struct ID3D11RasterizerState; struct ID3D11SamplerState; struct ID3D11Query;
typedef unsigned int UINT;
