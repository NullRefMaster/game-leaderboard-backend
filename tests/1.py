from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
from Crypto.Util.Padding import pad, unpad
from os import environ
import requests

def encrypt_data(key, data):
    iv = get_random_bytes(16)
    cipher = AES.new(key, AES.MODE_CBC, iv)
    ciphertext = cipher.encrypt(pad(data, AES.block_size))
    return iv + ciphertext

def decrypt_data(key, data):
    iv = data[:16]
    ciphertext = data[16:]
    cipher = AES.new(key, AES.MODE_CBC, iv)
    plaintext = unpad(cipher.decrypt(ciphertext), AES.block_size)
    return plaintext

def main():
    server_address = ('localhost', 3000)
    key = bytes.fromhex(environ["KEY"])
    plaintext = open("plaintext.txt", "rb").read()
    encrypted_data = encrypt_data(key, plaintext)
    decrypted_data = decrypt_data(key, encrypted_data)
    assert decrypted_data == plaintext
    response = requests.post(f"http://{server_address[0]}:{server_address[1]}/leaderboard/1", data=encrypted_data)
    print(response.status_code, response.text)

if __name__ == "__main__":
    main()

