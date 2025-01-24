document.addEventListener("htmx:beforeRequest", handleBeforeRequest);
document.addEventListener("htmx:afterRequest", handleAfterRegisterRequest);
document.addEventListener("htmx:responseError", handleRegistrationServerError);
document.addEventListener("htmx:load", handleRegisterLoad);

function handleRegistrationServerError(event) {
    const registrationFormId = "registration-form";
    if (event && event.target.id !== registrationFormId) {
        return;
    }

    const errorContainer = document.getElementById("server-error");
    const errorMessage = "An unexpected error occurred. Please try again.";

    displayErrorMessage(errorContainer, errorMessage);
}

function handleBeforeRequest(event) {
    const inputField = event.target;
    if (!inputField) return;

    const validationMap = {
        "register-username": validateUsername,
        "register-email": validateEmail,
    };

    const validateField = validationMap[inputField.id];
    if (validateField && !validateField()) {
        event.preventDefault(); // Prevent HTMX request if frontend validation fails
        return;
    }

    validateRegisterForm(event);
}

function handleAfterRegisterRequest(event) {
    const registrationFormId = "registration-form";

    if (event && event.target.id !== registrationFormId) {
        return;
    }

    closeAuthPopup(event);
}

function handleRegisterLoad() {
    document
        .getElementById("register-username")
        ?.removeEventListener("blur", validateUsername);
    document
        .getElementById("register-email")
        ?.removeEventListener("blur", validateEmail);
    document
        .getElementById("register-password")
        ?.removeEventListener("blur", validatePassword);
    document
        .getElementById("register-password2")
        ?.removeEventListener("blur", validatePasswordMatch);
    document
        .getElementById("register-profile_picture")
        ?.removeEventListener("change", validateProfilePicture);
    document
        .getElementById("register-username")
        ?.addEventListener("blur", validateUsername);
    document
        .getElementById("register-email")
        ?.addEventListener("blur", validateEmail);
    document
        .getElementById("register-password")
        ?.addEventListener("blur", validatePassword);
    document
        .getElementById("register-password2")
        ?.addEventListener("blur", validatePasswordMatch);
    document
        .getElementById("register-profile_picture")
        ?.addEventListener("change", validateProfilePicture);
}

function validateRegisterForm(event) {
    if (!isRegisterFormEventTarget(event) || !shouldValidate(event)) {
        return;
    }

    clearErrors();

    if (
        !validateUsername() ||
        !validateEmail() ||
        !validatePassword() ||
        !validatePasswordMatch() ||
        !validateProfilePicture()
    ) {
        event.preventDefault();
    }
}

function isRegisterFormEventTarget(event) {
    const form = document.getElementById("registration-form");

    return event.target === form;
}

function shouldValidate(event) {
    const target = event.target;

    return target?.matches("form[data-validate]");
}

function clearErrors() {
    document.querySelectorAll(".error-message").forEach((el) => {
        el.textContent = "";
        el.classList.add("hidden");
    });
}

function validateUsername() {
    const username = document.getElementById("register-username").value;
    const usernameError = document.getElementById("username-error");

    if (username.length < 3) {
        usernameError.textContent = "Username must be at least 3 characters.";
        usernameError.classList.remove("hidden");
        return false;
    }
    if (username.length > 12) {
        usernameError.textContent =
            "Username must be maximum of 12 characters long.";
        usernameError.classList.remove("hidden");
        return false;
    }

    usernameError.classList.add("hidden");
    return true;
}

function validateEmail() {
    const email = document.getElementById("register-email").value;
    const emailError = document.getElementById("email-error");
    const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailPattern.test(email)) {
        emailError.textContent = "Please enter a valid email address.";
        emailError.classList.remove("hidden");
        return false;
    }

    emailError.classList.add("hidden");
    return true;
}

function validatePassword() {
    const password = document.getElementById("register-password").value;
    const passwordError = document.getElementById("password-error");
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
    const password = document.getElementById("register-password").value;
    const password2 = document.getElementById("register-password2").value;
    const password2Error = document.getElementById("password2-error");
    if (password !== password2) {
        password2Error.textContent = "Passwords do not match.";
        password2Error.classList.remove("hidden");
        return false;
    }

    password2Error.classList.add("hidden");
    return true;
}

function validateProfilePicture() {
    const profilePicture = document.getElementById("register-profile_picture")
        .files[0];
    const profilePictureError = document.getElementById(
        "profile-picture-error"
    );

    if (profilePicture && profilePicture.size > 10 * 1024 * 1024) {
        profilePictureError.textContent = "Profile picture must be under 10MB.";
        profilePictureError.classList.remove("hidden");
        return false;
    }

    profilePictureError.classList.add("hidden");
    return true;
}

function previewRegisterImage() {
    const input = document.getElementById("register-profile_picture");
    const preview = document.getElementById("imagePreview");

    if (!input || !preview) {
        return;
    }

    const file = input.files[0];

    if (!file) {
        preview.src = "/static/images/anonymous_profile_picture.jpg";
        return;
    }

    const reader = new FileReader();
    reader.onload = function (e) {
        preview.src = e.target.result;
    };

    reader.readAsDataURL(file);
}
