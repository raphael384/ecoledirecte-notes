const { invoke } = window.__TAURI__.tauri;


let note_set = await invoke("get");

// Variables pour le premier conteneur (left)
const average = document.querySelector('.note-container.left .average');
const evolution_span = document.querySelector('.note-container.left .evolution');
const class_average = document.querySelector('.note-container.left .class-average');

// Variables pour le deuxième conteneur (right)
const coef_average = document.querySelector('.note-container.right .average');
const coef_evolution_span = document.querySelector('.note-container.right .evolution');
const coef_class_average = document.querySelector('.note-container.right .class-average');


average.innerHTML = Math.round(note_set.average * 100) / 100;
class_average.innerHTML = Math.round(note_set.class_average * 100) / 100;

coef_average.innerHTML = note_set.coef_average;
coef_class_average.innerHTML = note_set.coef_class_average;

let evolution = Math.round(note_set.evolution * 100) / 100;
let coef_evolution = Math.round(note_set.coef_evolution * 100) / 100;

if (evolution < 0) {
  evolution_span.style.color = "#ee5159";
  evolution = "- " + Math.abs(evolution) + "%"
} else if(evolution >= 0) {
  evolution_span.style.color = "#a019b7";
  evolution = "+ " + evolution + "%"
}

evolution_span.innerHTML = evolution;

if (coef_evolution < 0) {
  coef_evolution_span.style.color = "#ee5159";
  coef_evolution = "- " + Math.abs(coef_evolution) + "%"
} else if(coef_evolution >= 0) {
  coef_evolution_span.style.color = "#a019b7";
  coef_evolution = "+ " + coef_evolution + "%"
}

coef_evolution_span.innerHTML = coef_evolution;

