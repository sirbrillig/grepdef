export function queryDbTS(): string {}
export function queryDbTSFake(): string {}

const makeQueryTS: MyFunc = (): string => {};
const makeQueryTSFake: MyFunc = (): string => {};

function parseQueryTS(): string {
	const buildParserTS: MyFunc = (): string => {};
	const buildParserTSFake: MyFunc = (): string => {};
}

function parseQueryTSFake: MyFunc(): string {
}

const objectWithFunctionShorthandTS = {
	shorthandFunctionTS(): string {
		return 'hi';
	},
	shorthandFunctionTSFake(): string {
		return 'hi';
	}
};

const objectWithFunctionLonghandTS = {
	longhandFunctionTS: function(): string {
		return 'hi';
	},
	longhandFunctionTSFake: function(): string {
		return 'hi';
	}
};

const objectWithArrowFunctionLonghandTS = {
	longhandArrowFunctionTS: (): string => {
		return 'hi';
	},
	longhandArrowFunctionTSFake: (): string => {
		return 'hi';
	}
};

const objectWithPropertyLonghandTS = {
	longhandPropertyTS: 'hello',
	longhandPropertyTSFake: 'hello',
};

const objectWithPropertyShorthandTS = {
	shorthandPropertyTS,
	shorthandPropertyTSFake,
};

const arrayDef = [
	longhandFunctionTS,
	longhandArrowFunctionTS,
	longhandPropertyTS,
	shorthandPropertyTS,
];

interface AnInterface {
	foo: string;
}

type AType = 'something' | 'somethingelse';

/**
 * @typedef {object} TypeDefObject
 * @property {number} loaded Number of bytes already transferred
 * @property {number} total  Total number of bytes to transfer
 */

/**
 * @typedef TypeDefSimple
 * @property {number} loaded Number of bytes already transferred
 * @property {number} total  Total number of bytes to transfer
 */
