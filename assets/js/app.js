function filterEventsByCategory(category) {
    document.querySelectorAll(".event_card").forEach((card) => {
        const show = !category || card.dataset.category === category;
        card.style.display = show ? "" : "none";
    });
}