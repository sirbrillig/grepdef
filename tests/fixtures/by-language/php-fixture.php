<?php
function queryDb() {}

$makeQuery = function() {};

function parseQuery() {
	function buildParser() {};
	echo $makeQuery;
}

class Foo {
}

trait Bar {
}

interface Zoom {
}

enum MyEnum {
}

class MyClass {
	public function doSomething(): Foo {
	}
	public function doSomethingAbsolute(): \Home\Foo {
	}
}

const GLOBALCONSTANT = 77;
define( 'GLOBALDEFINESINGLE', 42 );
define( "GLOBALDEFINEDOUBLE", 43 );

class ClassWithConstant {
	public const MYCONSTANT = 5;

	public function echoConstant(): void {
		echo self::MYCONSTANT;
		echo GLOBALCONSTANT;
		echo GLOBALDEFINE;
	}
}
