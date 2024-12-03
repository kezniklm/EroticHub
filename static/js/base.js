// Age verification logic
document.addEventListener("DOMContentLoaded", function () {
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
});