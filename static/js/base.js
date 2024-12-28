document.addEventListener("DOMContentLoaded", function () {
    initializeAgeVerification();
    setupAccountPopup();
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

function setupAccountPopup( triggerId = "account-icon", popupId = "account-popup") {
    const triggerElement = document.getElementById(triggerId);
    const popupElement = document.getElementById(popupId);

    if (!triggerElement || !popupElement) {
        return;
    }

    const togglePopup = (event) => {
        event.preventDefault();
        popupElement.classList.toggle("hidden");
    };

    const closePopupIfOutsideClick = (event) => {
        if (!popupElement.contains(event.target) && !triggerElement.contains(event.target)) {
            popupElement.classList.add("hidden");
        }
    };

    triggerElement.addEventListener("click", togglePopup);
    document.addEventListener("click", closePopupIfOutsideClick);
}
