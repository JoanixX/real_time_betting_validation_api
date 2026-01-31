document.addEventListener("DOMContentLoaded", () => {
  const form = document.getElementById("bet-form");
  const statusDot = document.querySelector(".status-dot");
  const statusText = document.querySelector(".status-text");
  const formMessage = document.getElementById("form-message");
  const submitBtn = document.getElementById("submit-btn");
  const btnText = submitBtn.querySelector(".btn-text");
  const loader = submitBtn.querySelector(".loader-small");

  const API_BASE = window.API_URL || "";

  // 1. Verificación de Salud (Health Check)
  async function checkHealth() {
    try {
      const response = await fetch(`${API_BASE}/health_check`);
      if (response.ok) {
        statusDot.classList.add("online");
        statusText.textContent = "Validation API Online";
      } else {
        throw new Error();
      }
    } catch (error) {
      statusDot.classList.remove("online");
      statusText.textContent = "API Disconnected";
    }
  }

  checkHealth();
  setInterval(checkHealth, 30000);

  // 2. Envío del Formulario
  form.addEventListener("submit", async (e) => {
    e.preventDefault();

    // Reiniciar estados
    formMessage.textContent = "";
    formMessage.className = "message";
    btnText.style.display = "none";
    loader.hidden = false;
    submitBtn.disabled = true;

    // Obtener valores manuales para construir JSON
    const userId = document.getElementById("user_id").value;
    const matchId = document.getElementById("match_id").value;
    const amount = parseFloat(document.getElementById("amount").value);
    const odds = parseFloat(document.getElementById("odds").value);

    const payload = {
      user_id: userId,
      match_id: matchId,
      amount: amount,
      odds: odds,
    };

    try {
      const response = await fetch(`${API_BASE}/bets`, {
        method: "POST",
        body: JSON.stringify(payload),
        headers: {
          "Content-Type": "application/json",
        },
      });

      if (response.ok) {
        const data = await response.json();
        formMessage.textContent = `Bet Validated! Ticket: ${data.user_id.slice(0, 8)}...`;
        formMessage.classList.add("success");
        // No reseteamos todo el form para facilitar testing rapido
      } else {
        formMessage.textContent = `Rejected: ${response.statusText}`;
        formMessage.classList.add("error");
      }
    } catch (error) {
      console.error(error);
      formMessage.textContent = "Network Error";
      formMessage.classList.add("error");
    } finally {
      btnText.style.display = "inline-block";
      loader.hidden = true;
      submitBtn.disabled = false;
    }
  });

  // Efecto parallax sutil en las burbujas de fondo
  document.addEventListener("mousemove", (e) => {
    const x = e.clientX / window.innerWidth;
    const y = e.clientY / window.innerHeight;

    document.querySelectorAll(".blob").forEach((blob, index) => {
      const speed = (index + 1) * 20;
      blob.style.transform = `translate(${x * speed}px, ${y * speed}px)`;
    });
  });
});
