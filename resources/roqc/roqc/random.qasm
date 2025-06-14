OPENQASM 2.0;
include "qelib1.inc";
qreg q[5];
cx q[1],q[0];
rz(0.012991963724792915) q[1];
cx q[0],q[1];
rz(0.8701084837231242) q[4];