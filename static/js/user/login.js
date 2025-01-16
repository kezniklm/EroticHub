document.addEventListener("htmx:afterRequest", handleAfterLoginRequest);
document.addEventListener("htmx:responseError", handleLoginServerError);

function handleAfterLoginRequest(event) {
    const loginFormId = "login-form";

    const shouldCloseLoginPopup =
        (event && event.target.id !== loginFormId) ||
        event?.detail.failed === true;

    if (shouldCloseLoginPopup) {
        return;
    }

    closeAuthPopup(event);
}

function handleLoginServerError(event) {
    const loginFormId = "login-form";
    if (event && event.target.id !== loginFormId) {
        return;
    }

    const errorContainer = document.getElementById("server-error");

    const errorMessage = (() => {
        const errorCode = event.detail.xhr.status;

        switch (true) {
            case errorCode >= 400 && errorCode < 500:
                return "Invalid username or password. Please try again.";
            case errorCode >= 500:
                return "Server error. Please try again later.";
            default:
                return "An unexpected error occurred. Please try again.";
        }
    })();

    displayErrorMessage(errorContainer, errorMessage);
}
