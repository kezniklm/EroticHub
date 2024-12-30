document.addEventListener("htmx:load", function () {
    initializeAgeVerification();
    initTooltips();
});


function initializeAgeVerification() {
    const popup = document.getElementById('age-verification-popup');
    const yesButton = document.getElementById('yes-button');
    const noButton = document.getElementById('no-button');

    if (!localStorage.getItem('ageVerified')) {
        popup.classList.add('active');
    }

    yesButton.addEventListener('click', function () {
        localStorage.setItem('ageVerified', 'true');
        popup.classList.remove('active');
    });

    noButton.addEventListener('click', function () {
        window.location.href = 'https://www.google.com';
    });
}

function initTooltips() {
    const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]');
    [...tooltipTriggerList].map(tooltipTriggerEl => {
        const tooltip = new bootstrap.Tooltip(tooltipTriggerEl);
        tooltipTriggerEl.addEventListener("click", function () {
            tooltip.hide();
        })
        return tooltip; 
    });
}