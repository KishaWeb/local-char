let selectedCharacter = null;
let currentChat = null;

function toggleSidebar() {
    const sidebar = document.querySelector(".sidebar");
    const btn = document.getElementById("menuBtn");

    sidebar.classList.toggle("open");

    if (sidebar.classList.contains("open")) {
        btn.innerText = "✕";
    } else {
        btn.innerText = "☰";
    }
}

function addMessage(role, text) {
    const chat = document.getElementById("chat");

    const div = document.createElement("div");
    div.className = "msg " + role;
    div.innerText = text;

    chat.appendChild(div);
    chat.scrollTop = chat.scrollHeight;
}

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

    if (data.length > 0) {
        selectedCharacter = data[0].id;
        selector.value = selectedCharacter;
    }
}

async function newChat() {
    const res = await fetch("/new_chat", {
        method: "POST"
    });

    const data = await res.json();

    currentChat = data.chat_id;

    document.getElementById("chat").innerHTML = "";

    await renderChatList();
    
    if (window.innerWidth <= 768) {
        document.querySelector(".sidebar").classList.remove("open");
        document.getElementById("menuBtn").innerText = "☰";
    }
}

async function loadChat(id) {
    currentChat = id;

    const res = await fetch(`/chat_history/${id}`);
    const data = await res.json();

    const chat = document.getElementById("chat");
    chat.innerHTML = "";

    data.forEach(m => {
        addMessage(
            m.role === "assistant" ? "ai" : "user",
            m.content
        );
    });

    if (window.innerWidth <= 768) {
        document.querySelector(".sidebar").classList.remove("open");
    }
}

async function pinChat(id) {
    await fetch(`/pin/${id}`, {
        method: "POST"
    });

    await renderChatList();
}

async function deleteChat(id) {
    await fetch(`/delete/${id}`, {
        method: "POST"
    });

    if (currentChat === id) {
        currentChat = null;
        document.getElementById("chat").innerHTML = "";
    }

    await renderChatList();
}

async function renderChatList() {
    const res = await fetch("/chats");
    const data = await res.json();

    const pinned = document.getElementById("pinnedChats");
    const normal = document.getElementById("normalChats");

    pinned.innerHTML = "";
    normal.innerHTML = "";

    data.forEach(chat => {
        const item = document.createElement("div");
        item.className = "chat-item";

        const top = document.createElement("div");
        top.className = "chat-top";

        const title = document.createElement("span");
        title.className = "chat-title";
        title.innerText = chat.title || "New chat";
        title.onclick = () => loadChat(chat.id);

        const actions = document.createElement("div");
        actions.className = "chat-actions";

        const pin = document.createElement("button");
        pin.innerText = chat.pinned ? "Unpin" : "Pin";

        pin.onclick = async (e) => {
            e.stopPropagation();
            await pinChat(chat.id);
        };

        const del = document.createElement("button");
        del.innerText = "Delete";

        del.onclick = async (e) => {
            e.stopPropagation();

            if (confirm("Delete this chat?")) {
                await deleteChat(chat.id);
            }
        };

        actions.appendChild(pin);
        actions.appendChild(del);

        top.appendChild(title);
        top.appendChild(actions);

        item.appendChild(top);

        if (chat.pinned) {
            pinned.appendChild(item);
        } else {
            normal.appendChild(item);
        }
    });
}

async function sendMessage() {
    const input = document.getElementById("message");
    const message = input.value.trim();

    if (!message) return;

    if (!selectedCharacter) {
        alert("No character selected");
        return;
    }

    if (!currentChat) {
        await newChat();
    }

    input.value = "";

    addMessage("user", message);

    const chat = document.getElementById("chat");

    const thinking = document.createElement("div");
    thinking.className = "msg ai";
    thinking.innerText = "Thinking...";

    chat.appendChild(thinking);
    chat.scrollTop = chat.scrollHeight;

    try {
        const res = await fetch("/chat", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                chat_id: currentChat,
                message: message,
                character_id: selectedCharacter
            })
        });

        if (!res.ok) {
            throw new Error(`HTTP ${res.status}`);
        }

        const data = await res.json();

        thinking.remove();

        addMessage("ai", data.response);

        await renderChatList();
    } catch (err) {
        console.error(err);

        thinking.remove();

        addMessage("ai", "Request failed.");
    }
}

function setCharacter(id) {
    selectedCharacter = id;
}

document.addEventListener("keydown", (e) => {
    if (
        e.key === "Enter" &&
        document.activeElement.id === "message"
    ) {
        sendMessage();
    }
});

window.onload = async () => {
    await loadCharacters();
    await renderChatList();
    await newChat();
};