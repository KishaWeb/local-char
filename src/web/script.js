console.log("script loaded");

window.sendMessage = async function () {
    const message =
        document.getElementById("message").value;

    const response = await fetch(
        "http://127.0.0.1:8080/v1/chat/completions",
        {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({
                model: "local-model",
                messages: [
                    {
                        role: "user",
                        content: message
                    }
                ]
            })
        }
    );

    const data = await response.json();

    document.getElementById("output").innerText =
        data.choices[0].message.content;
};