const wasm = require("./main.rs")

wasm.initialize({noExitRuntime: true}).then(function (Module) {
    // you can call module.cwrap here to get function wrappers for Rust functions
    const add = Module.cwrap('add', 'number', ['number', 'number', 'number']);
    console.log('Calling rust functions from javascript!');
    console.log(add(1, 2, 3));

    const counter_create = Module.cwrap("counter_create", "number", []);
    const counter_increment = Module.cwrap("counter_increment", "", ["number"]);
    const counter_decrement = Module.cwrap("counter_decrement", "", ["number"]);
    const counter_set = Module.cwrap("counter_set", "", ["number", "number"]);
    const counter_get = Module.cwrap("counter_get", "number", ["number"]);
    const counter_destroy = Module.cwrap("counter_destroy", "", ["number"]);

    const counter = counter_create();

    var x = document.querySelectorAll(".some-button");
    for (i = 0; i < x.length; i++) {
        x[i].addEventListener("click", function (event) {
            console.log("click");
            counter_increment(counter);
            console.log(this.getAttribute("data-x"));
            console.log(this.getAttribute("data-y"));
            this.textContent = "Button: " + counter_get(counter);
        });
    }
});

