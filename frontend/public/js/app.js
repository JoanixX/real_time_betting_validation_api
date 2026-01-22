document.addEventListener("DOMContentLoaded", () => {
  const form = document.getElementById("subscribe-form");
  const statusDot = document.querySelector(".status-dot");
  const statusText = document.querySelector(".status-text");
  const formMessage = document.getElementById("form-message");
  const submitBtn = document.getElementById("submit-btn");
  const btnText = submitBtn.querySelector(".btn-text");
  const loader = submitBtn.querySelector(".loader-small");

  const API_BASE = window.API_URL || "";

  // 1. Health Check
  async function checkHealth() {
    try {
      const response = await fetch(`${API_BASE}/health_check`);
      if (response.ok) {
        statusDot.classList.add("online");
        statusText.textContent = "API Systems Operational";
      } else {
        throw new Error();
      }
    } catch (error) {
      statusDot.classList.remove("online");
      statusText.textContent = "API connection error";
    }
  }

  checkHealth();
  setInterval(checkHealth, 30000);

  // 2. Form Submission
  form.addEventListener("submit", async (e) => {
    e.preventDefault();

    // Reset states
    formMessage.textContent = "";
    formMessage.className = "message";
    btnText.style.display = "none";
    loader.hidden = false;
    submitBtn.disabled = true;

    const formData = new URLSearchParams(new FormData(form));

    try {
      const response = await fetch(`${API_BASE}/subscriptions`, {
        method: "POST",
        body: formData,
        headers: {
          "Content-Type": "application/x-www-form-urlencoded",
        },
      });

      if (response.ok) {
        formMessage.textContent = "Successfully subscribed!";
        formMessage.classList.add("success");
        form.reset();
      } else {
        const errorData = await response.text();
        formMessage.textContent = `Error: ${response.statusText}`;
        formMessage.classList.add("error");
      }
    } catch (error) {
      formMessage.textContent = "Network error. Please try again.";
      formMessage.classList.add("error");
    } finally {
      btnText.style.display = "inline-block";
      loader.hidden = true;
      submitBtn.disabled = false;
    }
  });

  // Subtle parallax effect on blobs
  document.addEventListener("mousemove", (e) => {
    const x = e.clientX / window.innerWidth;
    const y = e.clientY / window.innerHeight;

    document.querySelectorAll(".blob").forEach((blob, index) => {
      const speed = (index + 1) * 20;
      blob.style.transform = `translate(${x * speed}px, ${y * speed}px)`;
    });
  });
});
