document.addEventListener(
    "htmx:beforeRequest",
    handleBeforeChangePasswordRequest
);
document.addEventListener("htmx:responseError", handleChangePasswordServerError);

function handleChangePasswordServerError(event) {
    const changePasswordFormId = "password-change-form";
    if (event && event.target.id !== changePasswordFormId) {
        return;
    }

    const error_code = event.detail.xhr.status;

    if (!error_code) {
        return;
    }

    const errorMessage = (() => {
        switch (error_code) {
            case 400:
                return event.detail.xhr.responseText;
            default:
                return "An unexpected error occurred. Please try again.";
        }
    })();

    const errorContainer = document.getElementById("change-password-error");

    displayErrorMessage(errorContainer, errorMessage);
}

function handleBeforeChangePasswordRequest(event) {
    validatePasswordChangeForm(event);
}

function validatePasswordChangeForm(event) {
    if(event && event.target && event.target.id !== "password-change-form"){
        return;
    }

    clearErrors();

    if (!validatePassword() || !validatePasswordMatch()) {
        event.preventDefault();
    }
}

function validatePassword() {
    const password = document.getElementById("new-password").value;
    const passwordError = document.getElementById("change-password-error");
    if (password.length < 8) {
        passwordError.textContent = "Password must be at least 8 characters.";
        passwordError.classList.remove("hidden");
        return false;
    }

    if (password.length > 128) {
        passwordError.textContent =
            "Password must be maximum of 12 characters long.";
        passwordError.classList.remove("hidden");
        return false;
    }

    passwordError.classList.add("hidden");
    return true;
}

function validatePasswordMatch() {
    const password = document.getElementById("new-password").value;
    const password2 = document.getElementById("new-password2").value;
    const passwordError = document.getElementById("change-password-error");
    if (password !== password2) {
        passwordError.textContent = "Passwords do not match.";
        passwordError.classList.remove("hidden");
        return false;
    }

    passwordError.classList.add("hidden");
    return true;
}
