import tensorflow as tf
import numpy as np
import msgpack
from tensorflow import keras
import os
import sys

input_dir = sys.argv[1]

mnist = tf.keras.datasets.mnist
(images_train, labels_train), (images_test, labels_test) = mnist.load_data()

for i in range(10000):
    x = images_test[i]
    y = labels_test[i]
    #print(y)
    x = x.flatten() / 255.
    x = x.astype(np.float32)

    # print(x.dtype, x.shape)
    np.save(os.path.join(input_dir, str(i)+'_'+str(y)+'.npy'), x)
