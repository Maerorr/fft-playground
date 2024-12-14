import numpy as np

class EnvelopeFollower:
    def __init__(self, att, rel, sr):
        self.att = np.exp(-1 / (att * sr * 1e-3))
        self.rel = np.exp(-1 / (rel * sr * 1e-3))
        self.env = 0

    def process(self, buffer):
        self.env = np.zeros(len(buffer))
        last_env = -100.0
        for i, xn in enumerate(buffer):
            cur_env = 0.0
            if xn > last_env:
                cur_env = self.att * (last_env - xn) + xn
            else:
                cur_env = self.rel * (last_env - xn) + xn
            last_env = cur_env
            self.env[i] = cur_env

    def get_envelope(self):
        return self.env