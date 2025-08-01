import hashlib;
from pydoc import cli
from random import randint, randbytes
from base64 import b64encode
import socket
import time
import sys
import os
import math

host = "127.0.0.1"
port = 7878
RECV_SIZE = 1024
POW_LIMIT = 50_000_000
leading_zeros = 6

if __name__ != '__main':
    client = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    client.connect((host, port))
    # actual = client.recv(RECV_SIZE)

    with open("text.txt") as f:
        content = [line.removesuffix(os.linesep) for line in f.readlines()]
    
    # Send POST command first
    message = "POST\n"
    print(f"Sending: {repr(message)}")
    client.send(message.encode('utf-8'))
    time.sleep(0.1)  # Small delay
    
    # Send the middle data together
    for (index, line) in enumerate(content):
        client.send((line+'\n').encode())
    time.sleep(0.1)  # Small delay
    
    # Send SUBMIT command last
    message = "SUBMIT\n"
    print(f"Sending: {repr(message)}")
    client.send(message.encode('utf-8'))
    time.sleep(0.1)  # Small delay
    
    actual = client.recv(RECV_SIZE)
    challenge = actual.decode('utf-8').strip()[11:]
    print(f"Server Challenge: {challenge}")

    counter = 0
    spinner_period = 10000
    spinner = "-\\|/"
    while counter < POW_LIMIT:
        if counter%spinner_period == 0:
            print('\rSolving Challenge: '+spinner[counter//spinner_period%len(spinner)], end='')

        prefix = b64encode(randbytes(randint(3, 100)))
        s = '\n'.join([prefix.decode('utf-8')] + content + [challenge, ""])
        h = hashlib.sha256(str.encode(s)).hexdigest()

        c = 0
        while c < len(h) and h[c] == '0':
            c += 1
        if c >= leading_zeros:
            print() # break spinner
            print(f"Sending: ACCEPTED {repr(prefix)}")
            # print(f"Combined: {repr(s)}")
            # print(f"HASH found: {repr(h)}")
            client.send(b'ACCEPTED ' + prefix + b'\n')
            
            # Wait for server response
            response = client.recv(RECV_SIZE)
            print(f"Server Response: {response.decode('utf-8')}")
            
            # time.sleep(1)
            exit(0)
            # client.close()

        counter += 1

    
    client.close()
