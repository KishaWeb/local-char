let selectedCharacter = null;
let currentChat = null;
let chats = {};

/* TOGGLE SIDEBAR */
function toggleSidebar() {
    document.querySelector(".sidebar").classList.toggle("open");
}

/* RENDER MESSAGE */
function addMessage(role, text) {
    const chat = document.getElementById("chat");

    const div = document.createElement("div");
    div.className = "msg " + role;
    div.innerText = text;

    chat.appendChild(div);
    chat.scrollTop = chat.scrollHeight;
}

/* LOAD CHARACTERS */
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

/* NEW CHAT */
async function newChat() {
    const res = await fetch("/new_chat", { method: "POST" });
    const data = await res.json();

    currentChat = data.chat_id;
    chats[currentChat] = [];

    document.getElementById("chat").innerHTML = "";

    await renderChatList();
}

/* LOAD CHAT FROM BACKEND */
async function loadChat(id) {
    currentChat = id;

    const res = await fetch(`/chat_history/${id}`);
    const data = await res.json();

    chats[id] = data;

    const chat = document.getElementById("chat");
    chat.innerHTML = "";

    data.forEach(m => {
        addMessage(
            m.role === "assistant" ? "ai" : "user",
            m.content
        );
    });
}

/* CHAT LIST */
async function renderChatList() {
    const res = await fetch("/chats");
    const data = await res.json();

    const list = document.getElementById("chatList");
    list.innerHTML = "";

    data.forEach(chat => {
        const div = document.createElement("div");
        div.className = "chat-item";

        div.innerText = chat.title;

        div.onclick = () => loadChat(chat.id);

        list.appendChild(div);
    });
}

/* SEND MESSAGE */
async function sendMessage() {
    const input = document.getElementById("message");
    const message = input.value;
    input.value = "";

    if (!message.trim()) return;

    if (!currentChat) await newChat();

    addMessage("user", message);

    const res = await fetch("/chat", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
            chat_id: currentChat,
            message,
            character_id: selectedCharacter
        })
    });

    const data = await res.json();

    addMessage("ai", data.response);

    await renderChatList();
}

/* ENTER TO SEND */
document.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && document.activeElement.id === "message") {
        sendMessage();
    }
});

/* INIT */
window.onload = async () => {
    await loadCharacters();
    await renderChatList();
    await newChat();
};