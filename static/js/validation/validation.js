const CUSTOM_VALIDATORS = [
    {
        key: "file",
        fnc: fileValidator,
    }
]

const UNIT_MB = 1024 * 1024;
const UNIT_GB = UNIT_MB * 1024;

function setupValidations() {
    const elements = document.getElementsByClassName("validated");
    Array.from(elements).forEach(form => {
        if (form.tagName !== "FORM") {
            throw new Error(".validated class can be applied only to <form> elements")
        }

        const listener = (event) => validateForm(form, event);

        form.removeEventListener("submit", listener);
        form.addEventListener("submit", listener);

        attachInputListeners(form);
    })
}

function attachInputListeners(form) {
    const elements = form.elements;
    for (let i = 0; i < elements.length; i++) {
        const item = elements.item(i);
        const inputListener = () => validateInput(item);

        const customValidatorAttr = item.getAttribute("custom-validator");
        if (customValidatorAttr) {
            applyCustomValidator(item, customValidatorAttr);

            continue;
        }
        item.removeEventListener("keyup", inputListener);
        item.addEventListener("keyup", inputListener);
    }
}

/**
 * If elements includes `custom-validator` attribute, then custom validator is used
 * instead of HTML 5 Validation API
 *
 * Attribute `custom-validator-trigger` must be defined too.
 * @param input
 * @param validatorKey key of custom validator registered in {@link CUSTOM_VALIDATORS}
 */
function applyCustomValidator(input, validatorKey) {
    const validator = CUSTOM_VALIDATORS.find(
        validator => validator.key === validatorKey
    );

    const trigger = input.getAttribute("custom-validator-trigger");
    if (!trigger) {
        throw new Error("Custom validator trigger event is not defined for input " + input.name);
    }
    input.removeEventListener(trigger, validator.fnc);
    input.addEventListener(trigger, validator.fnc);
}

function validateForm(form, event) {
    if (!form.checkValidity()) {
        event.preventDefault();
        event.stopPropagation();
        event.stopImmediatePropagation();
    }
    for (let index = 0; index < form.elements.length; index++) {
        const input = form.elements.item(index);
        if (input.validity.valid) {
            continue;
        }
        validateInput(input);
    }
    markAsValidated(form);
}

function getErrorLabel(input) {
    const errorLabel = input.labels.values().find(label => label.className.includes("invalid-feedback"));
    if (!errorLabel) {
        console.warn(`Error label for input '${input.name}' is not defined`);
        return;
    }

    return errorLabel;
}

function validateInput(input) {
    const errorLabel = getErrorLabel(input);

    if (!errorLabel) {
        return;
    }
    if (errorLabel.textContent) {
        return
    }
    errorLabel.textContent = getErrorMessage(input);
}

function getErrorMessage(input) {
    const validity = input.validity;
    if (validity.valid) {
        return "";
    }

    if (input.required && validity.valueMissing) {
        return `Field is required!`;
    } else if (input.minLength && validity.tooShort) {
        return `Required length is ${input.minLength}`;
    } else if (input.maxLength && validity.tooLong) {
        return `Maximum length is ${input.maxLength}`;
    } else if (input.pattern && validity.badInput) {
        return `Field doesn't have required form`;
    }

    return "Validation failed!";
}

function fileValidator(event) {
    const input = event.target;
    let maxSize = input.getAttribute("max-size");
    if (!maxSize) {
        throw new Error("Max size value is not defined for input " + input.name);
    }

    let allowedSize = Number.MAX_VALUE;
    if (maxSize.endsWith("MB")) {
        allowedSize = Number(maxSize.slice(0, maxSize.indexOf("MB"))) * UNIT_MB;
    } else if (maxSize.endsWith("GB")) {
        allowedSize = Number(maxSize.slice(0, maxSize.indexOf("GB"))) * UNIT_GB;
    } else {
        throw new Error("Undefined file size unit!");
    }

    const files = input.files;
    let isError = false;
    for (let i =  0; i < files.length; i++) {
        if (files.item(i).size > allowedSize) {
            event.preventDefault();
            event.stopPropagation();
            event.stopImmediatePropagation();
            isError = true;
            break;
        }
    }

    if (!isError) {
        input.classList.add("is-valid");
        return;
    }

    const errorLabel = getErrorLabel(input);
    errorLabel.textContent = `Maximum allowed size of the file is ${maxSize}`;
    input.classList.add("is-invalid");
    input.value = "";
    markAsValidated(input.form);
}

function markAsValidated(form) {
    form.classList.add("was-validated");
}