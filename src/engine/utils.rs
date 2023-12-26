/// Interleave a buffer of samples into an output buffer.
pub fn interleave<T: Copy>(input: &[T], output: &mut [T], num_channels: usize) {
    debug_assert_eq!(input.len(), output.len());
    let num_samples = input.len() / num_channels;
    for sm in 0..num_samples {
        for ch in 0..num_channels {
            output[sm * num_channels + ch] = input[ch * num_samples + sm];
        }
    }
}

/// Deinterleave a buffer of samples into an output buffer
pub fn deinterleave<T: Copy>(input: &[T], output: &mut [T], num_channels: usize) {
    debug_assert_eq!(input.len(), output.len());
    let num_samples = input.len() / num_channels;
    for sm in 0..num_samples {
        for ch in 0..num_channels {
            output[ch * num_samples + sm] = input[sm * num_channels + ch];
        }
    }
}
