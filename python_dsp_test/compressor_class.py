# write a class that implements a dynamic range compressor that has both hard and soft knee options

import numpy as np
import matplotlib.pyplot as plt
from env_follower_class import EnvelopeFollower

class Compressor:
    def __init__(self, Th, R, W, ATT, REL, fs):
        self.Th = Th
        self.R = R
        self.UP_R = 1.0
        self.W = W
        self.ATT_MS = ATT
        self.ATT = np.exp(-1. / (fs * ATT * 1e-3))
        self.REL_MS = REL
        self.REL = np.exp(-1. / (fs * REL * 1e-3))
        self.fs = fs
        self.env_follower = EnvelopeFollower(ATT, REL, fs)

    def linear_to_db(self, x):
        return 20 * np.log10(np.abs(x) + 1e-8)

    def db_to_linear(self, db):
        return 10**(db / 20)

    def apply_compression(self, input_signal):
        self.env_follower.process(input_signal)
        env = self.env_follower.get_envelope()
        reduction = np.zeros(len(env))
        for i, x in enumerate(env):
            if (2 * (x - self.Th)) < -self.W:
                reduction[i] = x
            elif (2 * abs(x - self.Th)) <= self.W:
                reduction[i] = x + ((1.0 / self.R - 1.0)*(x - self.Th + self.W/2)**2) / (2.0 * self.W)
            else:
                reduction[i] = self.Th + (x - self.Th) / self.R
                
        return reduction - env

    
    def plot_compression(self, input_signal):
        #convert both input and compressed signals to dB
        input_signal_db = self.linear_to_db(input_signal)
        compressed_signal_db = self.linear_to_db(self.apply_compression(input_signal))
        

        # create x axis as time
        t = np.linspace(0, len(input_signal) / self.fs, len(input_signal))

        plt.figure(figsize=(10, 6))
        plt.plot(t, input_signal_db, label="Original Signal")
        plt.plot(t, compressed_signal_db, label="Compressed Signal")
        plt.xlabel("Time (s)")
        plt.ylabel("Amplitude")
        plt.title("Dynamic Range Compression")
        plt.legend()
        plt.show()

    def get_compressor_curve(self):
        buf = np.linspace(-100.0, 0.0, 1000)
        buf_o = np.linspace(-100.0, 0.0, 1000)

        for i, x in enumerate(buf):
            if (2 * (x - self.Th)) < -self.W:
                buf_o[i] = x
            elif (2 * abs(x - self.Th)) <= self.W:
                buf_o[i] = x + ((1.0 / self.R - 1.0)*(x - self.Th + self.W/2)**2) / (2.0 * self.W)
            else:
                buf_o[i] = self.Th + (x - self.Th) / self.R

        for i, x in enumerate(buf_o):
            if (2 * (x - self.Th)) < -self.W:
                buf_o[i] = self.Th + (x - self.Th) / self.UP_R
            elif (2 * abs(x - self.Th)) <= self.W:
                buf_o[i] = x - ((1.0 / self.UP_R - 1.0)*(x - self.Th - self.W/2)**2) / (2.0 * self.W)
            else:
                buf_o[i] = x
        return buf_o
