import numpy as np
import matplotlib.pyplot as plt

# Compressor parameters
T = -10.0  # Threshold in dB
R = 4.0    # Ratio
W = 6.0    # Knee width in dB
ATT = 5.0  # Attack time in ms
REL = 50.0 # Release time in ms
fs = 48000 # Sampling rate
duration = 2.0  # Duration of the sine wave (seconds)
f = 440.0  # Frequency of the sine wave (Hz)

# Generate the sine wave with block amplitude variation
t = np.linspace(0, duration, int(fs * duration), False)
sine_wave = np.sin(2 * np.pi * f * t)

# Create block amplitude changes in dB (convert to linear scale)
amplitude_blocks_db = [-20, -5, -10, 0]  # Amplitude in dB for each block (half-second each)
amplitude_blocks = [10**(db / 20) for db in amplitude_blocks_db]

# Assign block amplitudes
block_size = int(fs * 0.5)  # Each block lasts 0.5 seconds
amplitude_variation = np.zeros_like(sine_wave)

for i, amp in enumerate(amplitude_blocks):
    start = i * block_size
    end = (i + 1) * block_size if (i + 1) * block_size < len(sine_wave) else len(sine_wave)
    amplitude_variation[start:end] = amp

# Apply the varying amplitude to the sine wave
sine_wave_with_blocks = sine_wave * amplitude_variation

# Convert to dB
def linear_to_db(x):
    return 20 * np.log10(np.abs(x) + 1e-8)

def db_to_linear(db):
    return 10**(db / 20)

# Dynamic range compression function
def apply_compression(input_signal, T, R, W, ATT, REL, fs):
    # Attack and release coefficients
    attack_coef = np.exp(-1.0 / (ATT * fs / 1000.0))
    release_coef = np.exp(-1.0 / (REL * fs / 1000.0))

    output_signal = np.zeros_like(input_signal)
    gain = 1.0
    for i in range(len(input_signal)):
        x_db = linear_to_db(input_signal[i])
        
        # Knee compression calculation
        if x_db > T - W / 2.0:
            if x_db < T + W / 2.0:
                knee = (x_db - (T - W / 2.0)) / W
                gain_reduction = knee * ((x_db - T) / R - (x_db - T))
            else:
                gain_reduction = (x_db - T) / R - (x_db - T)
            desired_gain = db_to_linear(gain_reduction)
        else:
            desired_gain = 1.0

        # Attack and release smoothing
        if desired_gain < gain:
            gain = attack_coef * gain + (1 - attack_coef) * desired_gain
        else:
            gain = release_coef * gain + (1 - release_coef) * desired_gain

        # Apply gain
        output_signal[i] = input_signal[i] * gain

    return output_signal

# Apply compression to the sine wave
compressed_signal = apply_compression(sine_wave_with_blocks, T, R, W, ATT, REL, fs)

# Plot the original and compressed signals
plt.figure(figsize=(10, 6))
plt.plot(t, sine_wave_with_blocks, label="Original Signal with Block Amplitudes")
plt.plot(t, compressed_signal, label="Compressed Signal", linestyle="--")
plt.xlabel("Time [s]")
plt.ylabel("Amplitude")
plt.title("Dynamic Range Compression with Block Amplitude Changes")
plt.legend()
plt.show()