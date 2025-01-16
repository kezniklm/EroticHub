document.addEventListener("DOMContentLoaded", handleDOMContentLoaded);
document.addEventListener("htmx:afterSwap", handleAfterSwap);

function handleDOMContentLoaded() {
    initializeAgeVerification();
    setupAccountPopup();
    setupAuthPopup();
}

function handleAfterSwap(event) {
    if (event && event.target && isAllowedSwap(event.target)) {
        setupAccountPopup();
        setupAuthPopup();
    }
}

function isAllowedSwap(target) {
    const allowedClasses = ["account-container"];
    const allowedIds = ["account-container"];

    return (
        allowedClasses.some((className) =>
            target.classList.contains(className)
        ) || allowedIds.includes(target.id)
    );
}

function initializeAgeVerification() {
    const popup = document.getElementById("age-verification-popup");
    const yesButton = document.getElementById("yes-button");
    const noButton = document.getElementById("no-button");

    if (!localStorage.getItem("ageVerified")) {
        popup.classList.add("active");
    }

    yesButton.addEventListener("click", function () {
        localStorage.setItem("ageVerified", "true");
        popup.classList.remove("active");
    });

    noButton.addEventListener("click", function () {
        window.location.href = "https://www.google.com";
    });
}

function setupAccountPopup(
    triggerId = "account-icon",
    popupId = "account-popup"
) {
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
        if (
            !popupElement.contains(event.target) &&
            !triggerElement.contains(event.target)
        ) {
            popupElement.classList.add("hidden");
        }
    };

    triggerElement.removeEventListener("click", togglePopup);
    document.removeEventListener("click", closePopupIfOutsideClick);

    triggerElement.addEventListener("click", togglePopup);
    document.addEventListener("click", closePopupIfOutsideClick);
}

function setupAuthPopup() {
    const accountPopup = document.getElementById("account-popup");
    const authPopup = document.getElementById("auth-popup");

    function handleAccountPopupClick(e) {
        const action = e.target.closest(".popup-item");
        if (action) {
            const actionLabel = action
                .querySelector(".popup-label")
                .textContent.trim();
            switch (actionLabel) {
                case "Sign Up":
                case "Log In":
                    showAuthPopup();
                    break;
                case "Liked Videos":
                    // Add logic for liked videos if needed
                    break;
            }
        }
    }

    function handleAuthPopupClick(e) {
        if (e.target === authPopup) {
            closeAuthPopup();
        }
    }

    accountPopup.removeEventListener("click", handleAccountPopupClick);
    authPopup.removeEventListener("click", handleAuthPopupClick);

    accountPopup.addEventListener("click", handleAccountPopupClick);
    authPopup.addEventListener("click", handleAuthPopupClick);
}

function showAuthPopup() {
    const authPopup = document.getElementById("auth-popup");
    authPopup.classList.remove("hidden");
}

function closeAuthPopup(event) {
    const excludedTargets = [
        document.querySelector(".auth-popup"),
        document.querySelector(".popup-item"),
        document.querySelector("#register-username"),
        document.querySelector("#register-email"),
    ];

    if (
        event &&
        event.target &&
        excludedTargets.some(
            (target) =>
                target &&
                (target === event.target || target.contains(event.target))
        )
    ) {
        return;
    }

    const authPopup = document.getElementById("auth-popup");
    authPopup.classList.add("hidden");
}

function displayErrorMessage(container, message) {
    if (!container) {
        console.error("Error container element not found.");
        return;
    }
    container.textContent = message;
    container.classList.remove("hidden");
}
