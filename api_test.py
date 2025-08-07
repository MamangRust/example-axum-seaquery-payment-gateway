import requests
import json
import time
from datetime import datetime, timezone

# Konfigurasi
BASE_URL = "http://localhost:5000"  # Pastikan API jalan
HEADERS = {"Content-Type": "application/json"}

# Data pengguna
SENDER = {
    "firstname": "Alice",
    "lastname": "Smith",
    "email": f"alice_{int(time.time())}@example.com",
    "password": "password123",
    "confirm_password": "password123",
}

RECEIVER = {
    "firstname": "Bob",
    "lastname": "Johnson",
    "email": f"bob_{int(time.time())}@example.com",
    "password": "password123",
    "confirm_password": "password123",
}

# Token dan ID
AUTH_TOKEN_SENDER = None
AUTH_TOKEN_RECEIVER = None
SENDER_USER_ID = None
RECEIVER_USER_ID = None
SENDER_SALDO_ID = None
TOPUP_ID = None
TRANSFER_ID = None
WITHDRAW_ID = None


def print_response(res):
    print(f"Status: {res.status_code}")
    try:
        resp_json = res.json()
        print("Response:", json.dumps(resp_json, indent=2))
        return resp_json
    except Exception as e:
        print("Response:", res.text)
    print("-" * 60)
    return None


# 1. Register User
def register_user(user_data):
    print(f"\n=== 📝 Register: {user_data['email']} ===")
    url = f"{BASE_URL}/api/auth/register"
    payload = {
        "firstname": user_data["firstname"],
        "lastname": user_data["lastname"],
        "email": user_data["email"],
        "password": user_data["password"],
        "confirm_password": user_data["confirm_password"],
    }
    res = requests.post(url, headers=HEADERS, json=payload)
    data = print_response(res)
    if res.status_code == 200 and data and "data" in data:
        return data["data"].get("id")
    return None


# 2. Login User
def login_user(email, password):
    print(f"\n=== 🔐 Login: {email} ===")
    url = f"{BASE_URL}/api/auth/login"
    payload = {"email": email, "password": password}
    res = requests.post(url, headers=HEADERS, json=payload)
    data = print_response(res)
    if res.status_code == 200 and data and "data" in data:
        return data["data"]
    return None


# 3. Get Me
def get_me(token):
    print(f"\n=== 👤 GET /api/auth/me ===")
    headers = {**HEADERS, "Authorization": f"Bearer {token}"}
    res = requests.get(f"{BASE_URL}/api/auth/me", headers=headers)
    data = print_response(res)
    if res.status_code == 200 and data and "data" in data:
        return data["data"].get("id")
    return None


# 4. Create Saldo
def create_saldo(token, user_id, balance=1000000):
    print(f"\n=== 💰 Create Saldo for User ID: {user_id} | Balance: {balance} ===")
    url = f"{BASE_URL}/api/saldos"
    headers = {**HEADERS, "Authorization": f"Bearer {token}"}
    payload = {"user_id": user_id, "total_balance": balance}
    res = requests.post(url, headers=headers, json=payload)
    data = print_response(res)

    print("data saldo", data)

    if res.status_code == 201 and data and "data" in data:
        return data["data"].get("id")
    return None


# 5. Create Topup
def create_topup(token, user_id, amount=200000, method="gopay"):
    print(f"\n=== 📥 Create Topup for User ID: {user_id} | Amount: {amount} ===")
    url = f"{BASE_URL}/api/topups"
    headers = {**HEADERS, "Authorization": f"Bearer {token}"}
    payload = {
        "user_id": user_id,
        "topup_no": f"TOPUP{int(time.time())}",
        "topup_amount": amount,
        "topup_method": method,
    }
    res = requests.post(url, headers=headers, json=payload)
    data = print_response(res)
    if res.status_code == 201 and data and "data" in data:
        return data["data"].get("topup_id")
    return None


# 6. Create Transfer
def create_transfer(token, from_id, to_id, amount=50000):
    print(f"\n=== 🔄 Transfer: {from_id} → {to_id} | Amount: {amount} ===")
    url = f"{BASE_URL}/api/transfers"
    headers = {**HEADERS, "Authorization": f"Bearer {token}"}
    payload = {
        "transfer_from": from_id,
        "transfer_to": to_id,
        "transfer_amount": amount,
    }
    res = requests.post(url, headers=headers, json=payload)
    data = print_response(res)
    if res.status_code == 201 and data and "data" in data:
        return data["data"].get("transfer_id")
    return None


# 7. Create Withdraw
def create_withdraw(token, user_id, amount=50001):
    print(f"\n=== 📤 Withdraw by User ID: {user_id} | Amount: {amount} ===")
    url = f"{BASE_URL}/api/withdraws"
    headers = {**HEADERS, "Authorization": f"Bearer {token}"}

    # Waktu sekarang dalam format ISO 8601 UTC
    now_iso = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S") + "Z"
    payload = {"user_id": user_id, "withdraw_amount": amount, "withdraw_time": now_iso}
    res = requests.post(url, headers=headers, json=payload)
    data = print_response(res)
    if res.status_code == 201 and data and "data" in data:
        return data["data"].get("withdraw_id")
    return None


# # 8. Health Check
# def health_check():
#     print(f"\n=== 🩺 Health Checker ===")
#     url = f"{BASE_URL}/api/healthchecker"
#     res = requests.get(url)
#     print_response(res)
#     return res.status_code == 200


# --- 🔬 Jalankan Seluruh Alur ---
def run_full_test():
    global SENDER_USER_ID, RECEIVER_USER_ID
    global AUTH_TOKEN_SENDER, AUTH_TOKEN_RECEIVER
    global SENDER_SALDO_ID, TOPUP_ID, TRANSFER_ID, WITHDRAW_ID

    print('🚀 Starting Full API Test (All Responses: {"data": {"id": ...}})\n')

    # # 1. Cek API hidup
    # assert health_check(), "❌ Health check failed"
    # time.sleep(1)

    # 2. Register Sender
    SENDER_USER_ID = register_user(SENDER)
    assert SENDER_USER_ID, "❌ Register Sender GAGAL"
    print(f"✅ Sender dibuat: ID {SENDER_USER_ID}")
    time.sleep(1)

    # 3. Register Receiver
    RECEIVER_USER_ID = register_user(RECEIVER)
    assert RECEIVER_USER_ID, "❌ Register Receiver GAGAL"
    print(f"✅ Receiver dibuat: ID {RECEIVER_USER_ID}")
    time.sleep(1)

    # 4. Login Sender
    AUTH_TOKEN_SENDER = login_user(SENDER["email"], SENDER["password"])
    assert AUTH_TOKEN_SENDER, "❌ Login Sender GAGAL"
    time.sleep(1)

    # 5. Login Receiver
    AUTH_TOKEN_RECEIVER = login_user(RECEIVER["email"], RECEIVER["password"])
    assert AUTH_TOKEN_RECEIVER, "❌ Login Receiver GAGAL"
    time.sleep(1)

    # 6. Verifikasi /me
    me_sender = get_me(AUTH_TOKEN_SENDER)
    me_receiver = get_me(AUTH_TOKEN_RECEIVER)
    assert me_sender == SENDER_USER_ID, "❌ /me ID tidak cocok (sender)"
    assert me_receiver == RECEIVER_USER_ID, "❌ /me ID tidak cocok (receiver)"
    time.sleep(1)

    # 7. Buat saldo awal untuk sender
    SENDER_SALDO_ID = create_saldo(AUTH_TOKEN_SENDER, SENDER_USER_ID, 100000)
    assert SENDER_SALDO_ID, "❌ Gagal buat saldo"
    time.sleep(1)

    # # 8. Topup ke receiver
    TOPUP_ID = create_topup(AUTH_TOKEN_RECEIVER, RECEIVER_USER_ID, 300000, "shopeepay")
    assert TOPUP_ID, "❌ Topup GAGAL"
    time.sleep(1)

    # 9. Transfer dari sender ke receiver
    TRANSFER_ID = create_transfer(
        AUTH_TOKEN_SENDER, SENDER_USER_ID, RECEIVER_USER_ID, 50000
    )
    assert TRANSFER_ID, "❌ Transfer GAGAL"
    time.sleep(1)

    # 10. Withdraw oleh receiver (minimal 50.001)
    WITHDRAW_ID = create_withdraw(AUTH_TOKEN_RECEIVER, RECEIVER_USER_ID, 50001)
    assert WITHDRAW_ID, "❌ Withdraw GAGAL"

    # 🎉 Sukses!
    print('\n🎉🎉 SEMUA TEST BERHASIL! SEMUA RESPONSE = {"data": {"id": ...}}')
    print(f"🔑 Sender ID: {SENDER_USER_ID}")
    print(f"🔑 Receiver ID: {RECEIVER_USER_ID}")
    print(f"💰 Saldo ID: {SENDER_SALDO_ID}")
    print(f"📥 Topup ID: {TOPUP_ID}")
    print(f"🔄 Transfer ID: {TRANSFER_ID}")
    print(f"📤 Withdraw ID: {WITHDRAW_ID}")


if __name__ == "__main__":
    run_full_test()
