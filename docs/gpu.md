# OmniLang GPU & Tensor Extensions

## Overview

OmniLang includes experimental syntax for GPU kernel programming and tensor operations, designed to provide a clean path from prototyping to high-performance computing.

## GPU Kernel Syntax

### Basic Kernel Definition

```omni
// Define a GPU kernel
kernel void matrix_add(a: *Float, b: *Float, c: *Float, n: Int):
    let tid = get_global_id(0)
    if tid < n:
        c[tid] = a[tid] + b[tid]
```

### Built-in Kernel Functions

| Function | Description |
|----------|-------------|
| `get_global_id(dim)` | Global thread ID |
| `get_local_id(dim)` | Local thread ID within work group |
| `get_group_id(dim)` | Work group ID |
| `get_local_size(dim)` | Work group size |
| `get_num_groups(dim)` | Number of work groups |
| `barrier(flags)` | Synchronization barrier |
| `syncthreads()` | Synchronize all threads in group |

### Kernel Invocation

```omni
fn main() -> Int:
    // Launch kernel with 256 work items
    let global_size = 256
    let local_size = 64
    
    // Execute kernel
    matrix_add_kernel<<<global_size, local_size>>>(a, b, c, n)
    
    return 0
```

## Tensor Syntax

### Tensor Literals

```omni
// 1D tensor
let v = [1, 2, 3, 4, 5]

// 2D tensor (matrix)
let m = [[1, 2, 3],
         [4, 5, 6],
         [7, 8, 9]]

// 3D tensor
let t = [[[1, 2], [3, 4]],
         [[5, 6], [7, 8]]]
```

### Tensor Operations

```omni
fn main() -> Int:
    let a = [[1, 2], [3, 4]]
    let b = [[5, 6], [7, 8]]
    
    // Element-wise operations
    let sum = a + b
    let diff = a - b
    let prod = a * b
    
    // Matrix multiplication
    let result = a @ b
    
    // Reduction operations
    let total = sum(a)
    let average = mean(a)
    let maximum = max(a)
    
    return 0
```

### Broadcasting

OmniLang supports automatic broadcasting:

```omni
fn main() -> Int:
    let matrix = [[1, 2, 3],
                  [4, 5, 6]]
    let scalar = 10
    
    // Scalar broadcasts to all elements
    let result = matrix + scalar
    // Result: [[11, 12, 13],
    //          [14, 15, 16]]
    
    return 0
```

## GPU-Accelerated Functions

### `@device` Attribute

Mark functions for GPU execution:

```omni
@device
fn vector_add(a: [Float], b: [Float], n: Int) -> [Float]:
    let result = Vec::new()
    
    for i in 0..n:
        result.push(a[i] + b[i])
    
    return result

@device
fn matrix_multiply(a: [[Float]], b: [[Float]]) -> [[Float]]:
    // Efficient matrix multiplication
```

### `@parallel` Attribute

Auto-parallelize operations:

```omni
@parallel
fn process_pixels(image: [[Color]]) -> [[Color]]:
    // Each pixel processed in parallel
    // Similar to OpenMP or CUDA parallel for
```

## Example: Neural Network Layer

```omni
// Fully connected layer
struct DenseLayer {
    weights: Tensor<Float>,  // [input_size, output_size]
    biases: Tensor<Float>,   // [output_size]
}

impl DenseLayer {
    @device
    fn forward(input: Tensor<Float>) -> Tensor<Float>:
        // Matrix multiplication: output = input @ weights + biases
        let output = input @ self.weights
        let result = output + self.biases
        
        // Apply activation (ReLU)
        return relu(result)
    
    fn relu(x: Tensor<Float>) -> Tensor<Float>:
        return max(x, 0)
}

// Usage
fn main() -> Int:
    let layer = DenseLayer {
        weights: Tensor::random([784, 128]),
        biases: Tensor::zeros([128]),
    }
    
    let input = Tensor::random([784])
    let output = layer.forward(input)
    
    return 0
```

## Example: Convolution

```omni
@device
fn conv2d(input: Tensor<Float>, kernel: Tensor<Float>, stride: Int, padding: Int) -> Tensor<Float>:
    // 2D convolution operation
    // Used in CNNs for image processing
    
    let (h, w) = input.shape()
    let (kh, kw) = kernel.shape()
    
    let out_h = (h - kh + 2 * padding) / stride + 1
    let out_w = (w - kw + 2 * padding) / stride + 1
    
    let output = Tensor::zeros([out_h, out_w])
    
    // Apply convolution
    for y in 0..out_h:
        for x in 0..out_w:
            var sum = 0.0
            
            for ky in 0..kh:
                for kx in 0..kw:
                    let iy = y * stride + ky - padding
                    let ix = x * stride + kx - padding
                    
                    if iy >= 0 && iy < h && ix >= 0 && ix < w:
                        sum += input[iy, ix] * kernel[ky, kx]
            
            output[y, x] = sum
    
    return output
```

## Performance Considerations

### Memory Access Patterns

- **Coalesced Access**: Access memory sequentially for maximum bandwidth
- **Shared Memory**: Use local memory for frequently accessed data
- **Alignment**: Ensure data is properly aligned for vectorization

### Kernel Optimization Tips

1. **Minimize Branch Divergence**: Keep threads in a warp doing the same work
2. **Use Vector Types**: `float4`, `int4` for SIMD operations
3. **Occupancy**: Balance registers per thread vs. active threads

## Backend Support

### Current Status

| Backend | Status | Notes |
|---------|--------|-------|
| CPU | ✅ Available | Default backend |
| CUDA | ⚠️ Planned | Via LLVM NVPTX |
| OpenCL | ⚠️ Planned | Via LLVM AMDGPU |
| WebGPU | ⚠️ Planned | WASM compilation |

### Compilation Flow

```
OmniLang Source
       ↓
   Parser/AST
       ↓
Mid-Level IR (MLIR)
       ↓
   LLVM IR
       ↓
┌──────┴──────┐
│ CPU │ GPU   │
│ x86 │ NVPTX │
└──────┴──────┘
```

## Future Work

1. **Tensor Core Support**: Hardware acceleration for matrix operations
2. **Automatic Differentiation**: Built-in gradient computation
3. **JIT Compilation**: On-the-fly kernel generation
4. **Multi-GPU**: Distribute computation across devices

## Getting Started

```omni
// Simple GPU example
kernel void hello_world():
    printf("Hello from GPU thread %d!\n", get_global_id(0))

fn main() -> Int:
    // Execute on GPU
    hello_world<<<1024, 64>>>()
    
    return 0
```

This syntax is currently a **preview** - the parser accepts the syntax, but code generation for GPU backends is planned for a future release.
