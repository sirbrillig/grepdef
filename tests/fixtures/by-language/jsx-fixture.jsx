export function queryDb() {}
export function queryDbFake() {}

const makeQuery = () => {};
const makeQueryFake = () => {};

function parseQuery() {
	const buildParser = () => {};
	const buildParserFake = () => {};
}

function parseQueryFake() {
}

const objectWithFunctionShorthand = {
	shorthandFunction() {
		return 'hi';
	},
	shorthandFunctionFake() {
		return 'hi';
	}
};

const objectWithFunctionLonghand = {
	longhandFunction: function() {
		return 'hi';
	},
	longhandFunctionFake: function() {
		return 'hi';
	}
};

const objectWithArrowFunctionLonghand = {
	longhandArrowFunction: () => {
		return 'hi';
	},
	longhandArrowFunctionFake: () => {
		return 'hi';
	}
};

const objectWithPropertyLonghand = {
	longhandProperty: 'hello',
	longhandPropertyFake: 'hello',
};

const objectWithPropertyShorthand = {
	shorthandProperty,
	shorthandPropertyFake,
};

const arrayDef = [
	longhandFunction,
	longhandArrowFunction,
	longhandProperty,
	shorthandProperty,
];

