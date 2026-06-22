let currentChat = "chat_1";

async function sendMessage() {
    const input = document.getElementById("message");
    const msg = input.value;
    input.value = "";

    if (!msg) return;

    document.getElementById("output").innerText +=
        "You: " + msg + "\n";

    const res = await fetch("/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            chat_id: currentChat,
            message: msg
        })
    });

    const data = await res.json();

    document.getElementById("output").innerText +=
        "AI: " + data.response + "\n\n";
}

async function newChat() {
    const res = await fetch("/new_chat", {
        method: "POST"
    });

    const data = await res.json();
    currentChat = data.chat_id;

    document.getElementById("output").innerText +=
        "\n--- " + currentChat + " ---\n\n";
}