document.addEventListener("htmx:configRequest", checkUserDetailReadonly);
document.addEventListener("htmx:beforeRequest", handleBeforeRequest);
document.addEventListener("htmx:responseError", handleUpdateServerError);

function handleUpdateServerError(event) {
    const updateFormId = "user-edit-form";
    if (event && event.target.id !== updateFormId) {
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

    const errorContainer = document.getElementById("user-update-server-error");

    displayErrorMessage(errorContainer, errorMessage);
}

function enableEditing(event) {
    const usernameField = document.getElementById("user-username");
    const emailField = document.getElementById("user-email");
    const saveButton = document.getElementById("save-btn");
    const editButton = document.getElementById("edit-btn");

    usernameField.removeAttribute("readonly");
    emailField.removeAttribute("readonly");

    saveButton.classList.remove("hidden");
    editButton.classList.add("hidden");

    event.preventDefault();
}

function checkUserDetailReadonly(event) {
    const input = event.target;
    if (input.hasAttribute("readonly")) {
        event.preventDefault();
    }
}

function handleBeforeRequest(event) {
    handleBeforeUserUpdateRequest(event);
    handleBeforeProfilePictureUpdateRequest(event);
}

function validateProfilePicture(fileInput, errorMessage) {
    errorMessage.classList.add("hidden");
    errorMessage.textContent = "";

    if (!fileInput.files || fileInput.files.length === 0) {
        errorMessage.textContent =
            "Please select a profile picture before submitting.";
        errorMessage.classList.remove("hidden");
        return false;
    }

    const file = fileInput.files[0];
    const maxFileSize = 10 * 1024 * 1024;

    if (file.size > maxFileSize) {
        errorMessage.textContent =
            "The selected file exceeds the maximum size of 10MB.";
        errorMessage.classList.remove("hidden");
        return false;
    }

    if (!file.type.startsWith("image/")) {
        errorMessage.textContent = "Please select a valid image file.";
        errorMessage.classList.remove("hidden");
        return false;
    }

    return true;
}

function previewUserDetailImage() {
    const input = document.getElementById("change-profile_picture");
    const preview = document.getElementById("imagePreview");
    const errorMessage = document.getElementById("profile-picture-error");

    if (!input || !preview || !errorMessage) {
        return;
    }

    if (!validateProfilePicture(input, errorMessage)) {
        const originalPictureUrl = document.getElementById(
            "originalProfilePictureUrl"
        );
        const anonymousImageUrl =
            "/static/images/anonymous_profile_picture.jpg";

        if (!originalPictureUrl) {
            preview.src = anonymousImageUrl;
        } else {
            preview.src = originalPictureUrl.value;
        }
        return;
    }

    const file = input.files[0];
    const reader = new FileReader();
    reader.onload = function (e) {
        preview.src = e.target.result;
    };

    reader.readAsDataURL(file);
}

function handleBeforeProfilePictureUpdateRequest(event) {
    if (event.target.id !== "profile-picture-form") {
        return;
    }

    const fileInput = document.getElementById("change-profile_picture");
    const errorMessage = document.getElementById("profile-picture-error");

    if (!validateProfilePicture(fileInput, errorMessage)) {
        event.preventDefault();
    }
}

function handleBeforeUserUpdateRequest(event) {
    const inputField = event.target;
    if (!inputField) return;

    const validationMap = {
        "user-username": validateUsername,
        "user-email": validateEmail,
    };

    const validateField = validationMap[inputField.id];
    if (validateField && !validateField()) {
        event.preventDefault(); // Prevent HTMX request if frontend validation fails
    }
}

function validateUsername() {
    const username = document.getElementById("user-username").value;
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
    const email = document.getElementById("user-email").value;
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
