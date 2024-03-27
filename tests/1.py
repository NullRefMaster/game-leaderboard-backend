from Crypto.Cipher import ChaCha20_Poly1305
from Crypto.Random import get_random_bytes
from os import environ
import requests

def encrypt_data(key, data):
    nonce = get_random_bytes(12)
    cipher = ChaCha20_Poly1305.new(key=key, nonce=nonce)
    ciphertext, tag = cipher.encrypt_and_digest(data)
    return nonce + ciphertext + tag

def decrypt_data(key, data):
    cipher = ChaCha20_Poly1305.new(key=key, nonce=data[:12])
    plaintext = cipher.decrypt_and_verify(data[12:-16], data[-16:])
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

