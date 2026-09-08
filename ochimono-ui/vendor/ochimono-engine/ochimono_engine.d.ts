/* tslint:disable */
/* eslint-disable */

export class Game {
    free(): void;
    [Symbol.dispose](): void;
    advance(ticks: number): void;
    configure(gravity: boolean, handling: string): void;
    input(input: string): void;
    constructor(seed: string, mode: string, gravity: boolean, handling: string);
    redo(): boolean;
    release_inputs(): void;
    restore(snapshot: string): void;
    snapshot(): string;
    undo(): boolean;
    view(): string;
}
