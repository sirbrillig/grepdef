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
define( "GLOBALDEFINEDOUBLE_TWO", 41 ); // This is here to make sure that the other searches do not find it
define('GLOBALDEFINESINGLE_NOSPACE',40);
define ('GLOBALDEFINESINGLE_LEADSPACE',49);

class ClassWithConstant {
	public const MYCONSTANT = 5;

	public function echoConstant(): void {
		echo self::MYCONSTANT;
		echo GLOBALCONSTANT;
		echo GLOBALDEFINE;
	}
}

class ClassWithNonUniqueMethod1 {
	public function similarMethod() {}
}
class ClassWithNonUniqueMethod2 {
	public function similarMethod() {}
}
