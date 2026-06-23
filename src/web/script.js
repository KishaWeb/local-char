let selectedCharacter = "none";
let currentChat = null;

async function loadCharacters() {
    const res = await fetch("/characters");
    const data = await res.json();

    const selector = document.getElementById("characterSelect");

    selector.innerHTML = "";

    data.forEach(c => {
        const opt = document.createElement("option");
        opt.value = c.id;
        opt.innerText = c.id;
        selector.appendChild(opt);
    });

    selectedCharacter = data[0]?.id;
    selector.value = selectedCharacter;
}

async function newChat() {
    const res = await fetch("/new_chat", {
        method: "POST"
    });

    const data = await res.json();

    currentChat = data.chat_id;

    document.getElementById("output").innerText =
        "New chat: " + currentChat;
}

async function sendMessage() {
    const message = document.getElementById("message").value;

    if (!currentChat) {
        await newChat();
    }

    const res = await fetch("/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            chat_id: currentChat,
            message: message,
            character_id: selectedCharacter
        })
    });

    const data = await res.json();

    document.getElementById("output").innerText =
        data.response;
}

window.onload = async () => {
    await loadCharacters();
    await newChat();
};